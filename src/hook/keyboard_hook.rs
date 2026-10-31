use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use arc_swap::{ArcSwap, ArcSwapOption, Guard};
use windows::{
    Win32::{
        Foundation::{E_FAIL, E_UNEXPECTED},
        UI::WindowsAndMessaging::{EVENT_SYSTEM_DESKTOPSWITCH, EVENT_SYSTEM_FOREGROUND},
    },
    core::{Error, HRESULT, Result},
};

use crate::core::executor::{ExecutorHandle, Submitted, TargetWindow};
use crate::core::windows_actions::BindAction;
use crate::data::binding::Binding;
use crate::engine;
use crate::hook::health::HealthState;
use crate::hook::installer::{self, Installer};
use crate::hook::keyboard_hook_callback;
use crate::hook::main_thread::MainState;

/// What the hooks share with the other threads: the binding list, the hook
/// handles, the remapping switch, the action executor and the keyboard
/// hook's health. The engine is not here: it belongs to the main thread
/// (`hook::main_thread`, roadmap item 2.2).
pub struct KeyboardHook {
    /// Bindings list. Hot path reads via `ArcSwap::load()` which is
    /// effectively lock-free (no atomic CAS on the read side). Hot-reload
    /// updates the list with `update_bindings`, which atomically swaps the
    /// inner `Arc`. A macro queued or running on the action executor keeps
    /// its own `Arc` of its steps, so a swap never breaks it; the reload
    /// stops it through the executor's epoch instead (roadmap item 2.1).
    bindings: ArcSwap<Vec<Binding>>,
    hook_handle: AtomicPtr<std::ffi::c_void>,
    foreground_hook: AtomicPtr<std::ffi::c_void>,
    /// The `EVENT_SYSTEM_DESKTOPSWITCH` hook (roadmap item 2.3). Stored with
    /// `Release` in `install` and taken with `swap(AcqRel)` in `uninstall`,
    /// both on the main thread, like the other two handles.
    desktop_hook: AtomicPtr<std::ffi::c_void>,
    /// Whether remapping is on (`swch on`, `swch off`). While it is off the
    /// hooks stay installed and their callbacks pass every event untouched.
    ///
    /// Written only by [`switch_enabled`](Self::switch_enabled), on the main
    /// thread, between callbacks, with a `swap(AcqRel)`; read with
    /// `load(Acquire)` by the callbacks, on that thread too, and by `swch
    /// status` on an IPC worker. The flag publishes no other data: the
    /// engine state belongs to the main thread.
    enabled: AtomicBool,
    /// The action executor that `Launch`, `OpenUrl`, `Text`, `Paste` and
    /// `Sequence` go to (roadmap item 2.1). Stored only by the main thread,
    /// around the hooks' lifetime
    /// ([`attach_executor`](Self::attach_executor)); read with one lock-free
    /// load per slow action, like `bindings`.
    executor: ArcSwapOption<ExecutorHandle>,
    /// The keyboard callback's heartbeat, which the hook watchdog reads
    /// (roadmap item 2.4).
    health: HealthState,
}

impl KeyboardHook {
    /// Constructs an empty hook. Not `const` because `ArcSwap::from_pointee`
    /// requires a heap allocation; use a `LazyLock` for static initialization.
    pub fn new() -> Self {
        Self {
            bindings: ArcSwap::from_pointee(Vec::new()),
            hook_handle: AtomicPtr::new(std::ptr::null_mut()),
            foreground_hook: AtomicPtr::new(std::ptr::null_mut()),
            desktop_hook: AtomicPtr::new(std::ptr::null_mut()),
            enabled: AtomicBool::new(true),
            executor: ArcSwapOption::empty(),
            health: HealthState::new(),
        }
    }

    /// Atomically replace the binding list. Callable any number of times
    /// (this is the hot-reload entry point). The new list is sorted by
    /// key-count descending; among equal key counts, file order breaks a tie
    /// between two `keys` bindings of equal precedence (roadmap item 1.6).
    ///
    /// A reload also stops every macro running and every slow action still
    /// queued (roadmap item 2.1): they belong to the old list. The daemon
    /// reloads on its main thread, in `MainState::reload`, which resets the
    /// engine in the same step (roadmap item 2.3); at startup it stores the
    /// first list before the hooks are installed.
    pub fn update_bindings(&self, mut bindings: Vec<Binding>) {
        // A stable sort, so bindings with the same key count keep their
        // config order.
        engine::sort_by_specificity(&mut bindings);
        self.bindings.store(Arc::new(bindings));
        self.cancel_actions();
    }

    /// Attaches the action executor, or detaches it with `None`. Main
    /// thread: the executor is attached after it starts and before the
    /// hooks are installed, and detached after they are removed and before
    /// it stops.
    pub(crate) fn attach_executor(&self, executor: Option<Arc<ExecutorHandle>>) {
        self.executor.store(executor);
    }

    /// Hands a slow action to the action executor, on the thread that fired
    /// it, without waiting (see [`ExecutorHandle::submit`]). `Closed` when
    /// no executor is attached: at startup before it is, and at exit.
    pub(crate) fn submit(&self, action: BindAction, target: TargetWindow) -> Submitted {
        match &*self.executor.load() {
            Some(executor) => executor.submit(action, target),
            None => Submitted::Closed,
        }
    }

    /// Stops every macro running and every slow action still queued on the
    /// attached executor, if any; a step already running finishes first.
    pub(crate) fn cancel_actions(&self) {
        if let Some(executor) = &*self.executor.load() {
            executor.cancel_pending();
        }
    }

    /// Snapshot of the current binding list. The returned `Guard` derefs to
    /// `Arc<Vec<Binding>>` — callers can `.iter()` directly. The snapshot is
    /// stable for the guard's lifetime even if a reload happens concurrently.
    pub fn bindings(&self) -> Guard<Arc<Vec<Binding>>> {
        self.bindings.load()
    }

    /// The keyboard hook's health: the heartbeat the callback bumps.
    pub(crate) fn health(&self) -> &HealthState {
        &self.health
    }

    /// Whether remapping is on. See `switch_enabled`.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Flips the remapping switch and returns whether it changed. Only
    /// `MainState::set_enabled` calls it, and resets the engine with it: the
    /// `&mut MainState` it takes exists only on the main thread, inside
    /// `with_main`, so no other thread can flip the switch apart from the
    /// reset, between two events (roadmap item 2.2). The hooks stay
    /// installed either way; only the main thread installs and removes them
    /// (KS-01).
    pub(crate) fn switch_enabled(&self, enabled: bool, _main: &mut MainState) -> bool {
        self.enabled.swap(enabled, Ordering::AcqRel) != enabled
    }

    /// Installs the keyboard hook, the foreground hook and the
    /// desktop-switch hook.
    ///
    /// Main thread only: the thread that runs the main loop
    /// (`hook::main_thread`), because the hooks belong to the thread that
    /// installs them and are called through its message loop (KS-01).
    pub(crate) fn install(&self) -> Result<()> {
        self.install_with(&installer::SYSTEM)
    }

    /// [`install`](Self::install) through `installer`: the system's in the
    /// daemon, a fake in the tests (ruling S1).
    pub(crate) fn install_with(&self, installer: &impl Installer) -> Result<()> {
        if !self.hook_handle.load(Ordering::Acquire).is_null() {
            return Err(Error::new(
                E_UNEXPECTED,
                "keyboard hook is already installed",
            ));
        }

        let hook = installer.set_keyboard_hook()?;
        self.hook_handle.store(hook, Ordering::Release);
        self.health.set_hook_installed(true);

        // Foreground-change hook re-syncs modifier state after focus returns from
        // a higher-integrity window (Task Manager, UAC, lock screen, etc.) that
        // ate our key-up events, except for modifiers whose key-down the engine
        // swallowed. See keyboard_hook_callback::foreground_changed.
        let Some(we_hook) = installer.set_win_event_hook(
            EVENT_SYSTEM_FOREGROUND,
            Some(keyboard_hook_callback::foreground_changed),
        ) else {
            // The keyboard hook is already installed; tear it down to keep state
            // consistent with the failure being reported.
            let _ = self.uninstall_with(installer);
            return Err(Error::new(
                E_FAIL,
                "SetWinEventHook failed for EVENT_SYSTEM_FOREGROUND",
            ));
        };
        self.foreground_hook.store(we_hook, Ordering::Release);

        // A switch of the input desktop (a lock, Ctrl+Alt+Del, a UAC prompt
        // on the secure desktop, and back) is noted for the main loop, which
        // resets when keyboard input left this desktop (roadmap item 2.3).
        let Some(desktop_hook) = installer.set_win_event_hook(
            EVENT_SYSTEM_DESKTOPSWITCH,
            Some(keyboard_hook_callback::desktop_switched),
        ) else {
            let _ = self.uninstall_with(installer);
            return Err(Error::new(
                E_FAIL,
                "SetWinEventHook failed for EVENT_SYSTEM_DESKTOPSWITCH",
            ));
        };
        self.desktop_hook.store(desktop_hook, Ordering::Release);

        Ok(())
    }

    /// True while the daemon holds a keyboard hook handle. Windows removes a
    /// hook whose callback ran past `LowLevelHooksTimeout` without a word
    /// (KS-08), so this does not say that the hook is still called: the hook
    /// watchdog finds that out, from raw keyboard input (roadmap item 2.4).
    pub fn is_installed(&self) -> bool {
        !self.hook_handle.load(Ordering::Acquire).is_null()
    }

    /// Removes the three hooks. A WinEvent hook that cannot be removed is
    /// logged; the keyboard hook's failure is returned.
    ///
    /// Main thread only, like [`install`](Self::install):
    /// `UnhookWinEvent` fails on any other thread.
    pub(crate) fn uninstall(&self) -> Result<()> {
        self.uninstall_with(&installer::SYSTEM)
    }

    /// [`uninstall`](Self::uninstall) through `installer`. Each handle is
    /// taken out of its field before it is removed, so it is removed at most
    /// once, and a hook that is not installed is not asked about.
    pub(crate) fn uninstall_with(&self, installer: &impl Installer) -> Result<()> {
        let desktop_handle = self
            .desktop_hook
            .swap(std::ptr::null_mut(), Ordering::AcqRel);
        if !desktop_handle.is_null()
            && let Err(error) = installer.unhook_win_event(desktop_handle)
        {
            log::warn!("could not remove the desktop-switch hook: {error}");
        }

        let we_handle = self
            .foreground_hook
            .swap(std::ptr::null_mut(), Ordering::AcqRel);
        if !we_handle.is_null()
            && let Err(error) = installer.unhook_win_event(we_handle)
        {
            log::warn!("could not remove the foreground hook: {error}");
        }

        let handle = self
            .hook_handle
            .swap(std::ptr::null_mut(), Ordering::AcqRel);
        self.health.set_hook_installed(false);
        if !handle.is_null() {
            installer.unhook_keyboard(handle)?;
        }
        Ok(())
    }

    /// Replaces the keyboard hook with a new one, for the hook watchdog
    /// (roadmap item 2.4, KS-08). Main thread only, outside every callback:
    /// the main loop's drain serves the watchdog's request, so `install`,
    /// `uninstall` and this never overlap.
    ///
    /// - No keyboard hook installed (before `install`, or after `uninstall`
    ///   at exit or at the session end): `NotInstalled`, and no call.
    /// - Otherwise the new hook goes in first, at the head of the chain, and
    ///   only then the old one comes off. Windows calls a low-level hook only
    ///   inside a message call of this thread, and none happens between the
    ///   two calls, so every key event sees exactly one hook of ours: never
    ///   two, and never none.
    /// - A new hook that Windows refuses is an error, and the old one stays
    ///   installed and held: the daemon never has fewer hooks than before.
    pub(crate) fn reinstall_keyboard_hook(&self) -> Result<Reinstalled> {
        self.reinstall_keyboard_hook_with(&installer::SYSTEM)
    }

    /// [`reinstall_keyboard_hook`](Self::reinstall_keyboard_hook) through
    /// `installer`.
    pub(crate) fn reinstall_keyboard_hook_with(
        &self,
        installer: &impl Installer,
    ) -> Result<Reinstalled> {
        if self.hook_handle.load(Ordering::Acquire).is_null() {
            return Ok(Reinstalled::NotInstalled);
        }
        let new = installer.set_keyboard_hook()?;
        let previous = self.hook_handle.swap(new, Ordering::AcqRel);
        let old = match installer.unhook_keyboard(previous) {
            Ok(()) => OldHook::StillInstalled,
            Err(error) => OldHook::Gone(error.code()),
        };
        Ok(Reinstalled::Replaced { old })
    }
}

/// What [`KeyboardHook::reinstall_keyboard_hook`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reinstalled {
    /// No keyboard hook is installed, so none was installed either.
    NotInstalled,
    /// A new hook replaced the old one, which went as `old` says.
    Replaced { old: OldHook },
}

/// What became of the old keyboard hook in a reinstall.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OldHook {
    /// It was still installed, and is removed now.
    StillInstalled,
    /// Removing it failed with this error: Windows had removed it already,
    /// as it does after a callback that ran past `LowLevelHooksTimeout`.
    Gone(HRESULT),
}

impl fmt::Display for OldHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OldHook::StillInstalled => f.write_str("the old hook was still installed"),
            OldHook::Gone(code) => write!(
                f,
                "Windows had removed the old hook: {}",
                Error::from(*code)
            ),
        }
    }
}

impl Default for KeyboardHook {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        let _ = self.uninstall();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::core::executor::testing::{
        FakeRunner, Record, leaked_counters, next_record, wait_until,
    };
    use crate::core::executor::{Executor, QUEUE_CAPACITY, Stopped, Submitted, TargetWindow};
    use crate::core::windows_actions::BindAction;
    use crate::data::binding::Binding;
    use crate::data::key_combination::KeyCombination;
    use crate::data::key_spec::KeySpec;
    use crate::data::sequence::SequenceStep;
    use crate::data::trigger::Trigger;
    use crate::engine::{Decision, KeyEvent, TimerToken};
    use crate::hook::installer::{Call, FakeInstaller};
    use crate::hook::main_thread::{MainThread, with_main};
    use windows::Win32::Foundation::{E_ACCESSDENIED, ERROR_INVALID_HOOK_HANDLE};
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    /// Turns remapping on or off on `hook` through this thread's main
    /// state, as the main thread serves `swch on` and `off`, and returns
    /// whether the switch changed.
    fn set_enabled(hook: &KeyboardHook, enabled: bool) -> bool {
        with_main(|main| main.set_enabled(hook, enabled))
            .expect("this thread has a free main state")
    }

    fn three_bindings() -> Vec<Binding> {
        vec![
            Binding::new(KeyCombination::new(VK_CAPITAL), BindAction::SwitchLanguage),
            Binding::new(
                KeyCombination::new(VK_CAPITAL).with(VK_SHIFT),
                BindAction::ToggleCapsLock,
            ),
            Binding::new(
                KeyCombination::new(VK_A).with(VK_CONTROL).with(VK_SHIFT),
                BindAction::DoNothing,
            ),
        ]
    }

    #[test]
    fn bindings_sorted_by_specificity() {
        let hook = KeyboardHook::new();
        hook.update_bindings(three_bindings());

        let bindings = hook.bindings();
        assert_eq!(bindings[0].kind.key_count(), 3);
        assert_eq!(bindings[1].kind.key_count(), 2);
        assert_eq!(bindings[2].kind.key_count(), 1);
    }

    /// 30 distinct single keys, in the order a config would list them.
    const SINGLE_KEYS: [VIRTUAL_KEY; 30] = [
        VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12,
        VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F20, VK_F21, VK_F22, VK_F23,
        VK_F24, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5,
    ];

    #[test]
    fn equal_specificity_keeps_config_order() {
        // The sort must be stable: among bindings with the same key count,
        // the one listed first in the config is checked first (the file-order
        // tie-break between two `keys` bindings of equal precedence relies on
        // this, roadmap item 1.6).
        //
        // The list must be longer than 20 and not already sorted: for 20 or
        // fewer elements `sort_unstable*` falls back to insertion sort, which
        // happens to be stable, so a short list cannot tell the two apart.
        // A 2-key binding in the middle forces the sort to move elements.
        // With `sort_unstable_by_key` in `update_bindings` this test fails.
        let mut config: Vec<Binding> = SINGLE_KEYS
            .iter()
            .map(|&vk| Binding::new(KeyCombination::new(vk), BindAction::DoNothing))
            .collect();
        config.insert(
            15,
            Binding::new(
                KeyCombination::new(VK_A).with(VK_CONTROL),
                BindAction::SwitchLanguage,
            ),
        );
        assert!(config.len() > 20);

        let hook = KeyboardHook::new();
        hook.update_bindings(config);

        let bindings = hook.bindings();
        assert_eq!(bindings[0].kind.key_count(), 2);
        let single_keys: Vec<_> = bindings[1..]
            .iter()
            .map(|b| b.combination().unwrap().keys[0])
            .collect();
        assert_eq!(single_keys, SINGLE_KEYS.map(KeySpec::from));
    }

    #[test]
    fn update_bindings_replaces_list() {
        // Sanity check that ArcSwap actually swaps — a second update with
        // fewer bindings must shrink the observable list, and the new sort
        // must be applied (not just appended).
        let hook = KeyboardHook::new();
        hook.update_bindings(three_bindings());
        assert_eq!(hook.bindings().len(), 3);

        hook.update_bindings(vec![Binding::new(
            KeyCombination::new(VK_F13),
            BindAction::SwitchLanguage,
        )]);
        let snapshot = hook.bindings();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(
            snapshot[0].combination().unwrap().keys,
            vec![KeySpec::from(VK_F13)]
        );
    }

    #[test]
    fn update_bindings_can_clear_to_empty() {
        let hook = KeyboardHook::new();
        hook.update_bindings(three_bindings());
        hook.update_bindings(vec![]);
        assert_eq!(hook.bindings().len(), 0);
    }

    #[test]
    fn initial_bindings_are_empty() {
        let hook = KeyboardHook::new();
        assert_eq!(hook.bindings().len(), 0);
    }

    #[test]
    fn toggling_reports_changes_and_never_installs_a_hook() {
        let _main = MainThread::new().expect("the main thread is set up");
        let hook = KeyboardHook::new();
        assert!(hook.is_enabled(), "remapping starts on");
        assert!(!set_enabled(&hook, true), "already on");
        assert!(!hook.is_installed());
        assert!(set_enabled(&hook, false));
        assert!(!hook.is_enabled());
        assert!(!hook.is_installed());
        assert!(!set_enabled(&hook, false), "already off");
        assert!(!hook.is_installed());
        assert!(set_enabled(&hook, true));
        assert!(hook.is_enabled());
        assert!(!hook.is_installed(), "a toggle never installs a hook");
    }

    /// An [`engine::Env`] stopped at one instant with nothing held and no
    /// foreground window.
    struct QuietEnv(Instant);

    impl engine::Env for QuietEnv {
        fn now(&self) -> Instant {
            self.0
        }
        fn os_key_down(&self, _vk: VIRTUAL_KEY) -> bool {
            false
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn turning_remapping_off_drops_a_pending_hold_and_a_repeated_on_keeps_it() {
        // KS-31. No loop runs: the test hands the tokens back to the engine
        // itself, so nothing fires outside the assertions.
        let _main = MainThread::new().expect("the main thread is set up");
        let hook = KeyboardHook::new();
        hook.update_bindings(vec![
            Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)
                .with_trigger(Trigger::Hold(200))
                .with_allow_extra_modifiers(true),
        ]);
        let t0 = Instant::now();
        let key = |event: KeyEvent, at: Duration| {
            with_main(|main| {
                main.engine
                    .on_key(event, &hook.bindings(), &QuietEnv(t0 + at))
            })
            .expect("the main state is free")
        };
        let timer = |token: TimerToken| {
            with_main(|main| main.engine.on_timer(token)).expect("the main state is free")
        };

        let first = key(KeyEvent::down(VK_F13), Duration::ZERO).timers[0].token;
        assert!(!set_enabled(&hook, true));
        assert_eq!(
            timer(first).fire,
            [BindAction::DoNothing],
            "a toggle that changes nothing keeps the pending Hold"
        );

        let _ = key(KeyEvent::up(VK_F13), Duration::from_millis(300));
        let second = key(KeyEvent::down(VK_F13), Duration::from_millis(400)).timers[0].token;
        assert!(set_enabled(&hook, false));
        assert_eq!(
            timer(second),
            Decision::default(),
            "a Hold armed before remapping went off must not fire"
        );
    }

    /// A local hook with an attached executor whose runner only records,
    /// the executor, and the runner's records.
    fn hook_with_executor() -> (KeyboardHook, Executor, mpsc::Receiver<Record>) {
        let (tx, records) = mpsc::channel();
        let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        let hook = KeyboardHook::new();
        hook.attach_executor(Some(executor.handle()));
        (hook, executor, records)
    }

    fn macro_of(steps: Vec<SequenceStep>) -> BindAction {
        BindAction::Sequence(std::sync::Arc::new(steps))
    }

    /// Submits `[F13, Delay(10000), F14]` through `hook`, waits until F13
    /// ran and the macro waits in its Delay, runs `cancel`, and checks that
    /// the macro is gone within 100 ms and F14 never runs.
    fn a_running_macro_stops_within_100_ms(cancel: impl FnOnce(&KeyboardHook)) {
        let (hook, executor, records) = hook_with_executor();
        let handle = executor.handle();
        assert_eq!(
            hook.submit(
                macro_of(vec![
                    SequenceStep::PressKey(VK_F13),
                    SequenceStep::Delay(10_000),
                    SequenceStep::PressKey(VK_F14),
                ]),
                TargetWindow::NONE,
            ),
            Submitted::Queued
        );
        assert_eq!(next_record(&records).text(), "key F13");
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);

        let t0 = Instant::now();
        cancel(&hook);
        wait_until("the macro is gone", || handle.active_runs() == 0);
        let stopped = t0.elapsed();
        assert!(
            stopped < Duration::from_millis(100),
            "the macro stopped {stopped:?} later"
        );

        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_recv().is_err(), "F14 never runs");
    }

    #[test]
    fn swch_off_stops_a_running_macro_within_100_ms() {
        let _main = MainThread::new().expect("the main thread is set up");
        a_running_macro_stops_within_100_ms(|hook| {
            assert!(set_enabled(hook, false));
        });
    }

    #[test]
    fn a_reload_stops_a_running_macro_within_100_ms() {
        a_running_macro_stops_within_100_ms(|hook| hook.update_bindings(vec![]));
    }

    #[test]
    fn a_repeated_swch_on_keeps_the_running_macro() {
        let _main = MainThread::new().expect("the main thread is set up");
        let (hook, executor, records) = hook_with_executor();
        hook.submit(
            macro_of(vec![
                SequenceStep::PressKey(VK_F13),
                SequenceStep::Delay(300),
                SequenceStep::PressKey(VK_F14),
            ]),
            TargetWindow::NONE,
        );
        assert_eq!(next_record(&records).text(), "key F13");

        assert!(!set_enabled(&hook, true), "already on");

        assert_eq!(next_record(&records).text(), "key F14");
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_slow_action_without_an_executor_is_closed() {
        assert_eq!(
            KeyboardHook::new().submit(
                BindAction::OpenUrl("https://example.com".to_owned()),
                TargetWindow::NONE
            ),
            Submitted::Closed
        );
    }

    /// A hook that is never dropped: one that holds the handles of a fake
    /// would remove them through the system installer, which panics under
    /// test.
    fn leaked() -> &'static KeyboardHook {
        Box::leak(Box::new(KeyboardHook::new()))
    }

    /// A leaked hook installed through `fake`, whose calls are taken: the
    /// keyboard hook is handle 1, the two WinEvent hooks 2 and 3.
    fn installed(fake: &FakeInstaller) -> &'static KeyboardHook {
        let hook = leaked();
        hook.install_with(fake)
            .expect("the fake installs every hook");
        assert_eq!(
            fake.take_calls(),
            [
                Call::InstallKeyboard(1),
                Call::InstallWinEvent(2),
                Call::InstallWinEvent(3)
            ]
        );
        hook
    }

    /// Removes `hook`'s hooks through `fake` and returns the keyboard
    /// handle it removed.
    fn removed_keyboard_hook(hook: &KeyboardHook, fake: &FakeInstaller) -> Option<usize> {
        hook.uninstall_with(fake)
            .expect("the fake removes the keyboard hook");
        fake.take_calls().into_iter().find_map(|call| match call {
            Call::RemoveKeyboard(handle) => Some(handle),
            _ => None,
        })
    }

    #[test]
    fn a_reinstall_installs_the_new_hook_before_it_removes_the_old_one() {
        // Ruling S3, critique I2: a key event sees exactly one hook of
        // ours, never none, so the new hook goes in first, then the old one
        // comes off, and the hook keeps the new handle.
        let fake = FakeInstaller::new();
        let hook = installed(&fake);

        assert_eq!(
            hook.reinstall_keyboard_hook_with(&fake),
            Ok(Reinstalled::Replaced {
                old: OldHook::StillInstalled
            })
        );
        assert_eq!(
            fake.keyboard_counts(),
            [2, 1],
            "one keyboard hook was installed at every step, never none"
        );
        assert_eq!(
            fake.take_calls(),
            [Call::InstallKeyboard(4), Call::RemoveKeyboard(1)],
            "the new hook first, then the old one off"
        );
        assert_eq!(fake.keyboard_hooks(), [4], "one keyboard hook, the new one");
        assert_eq!(removed_keyboard_hook(hook, &fake), Some(4));
    }

    #[test]
    fn a_failed_reinstall_keeps_the_old_hook() {
        // Ruling S3: a new hook that Windows refuses leaves the old one
        // installed and in the hook, so the daemon never has fewer hooks
        // than before.
        let fake = FakeInstaller::new();
        let hook = installed(&fake);
        fake.refuse_next_keyboard_hook();

        let error = hook
            .reinstall_keyboard_hook_with(&fake)
            .expect_err("the fake refuses the new hook");
        assert_eq!(error.code(), E_ACCESSDENIED);
        assert_eq!(
            fake.take_calls(),
            [Call::KeyboardRefused],
            "the old hook is left alone"
        );
        assert_eq!(fake.keyboard_hooks(), [1]);
        assert!(hook.is_installed());
        assert_eq!(
            removed_keyboard_hook(hook, &fake),
            Some(1),
            "the hook still holds the old handle"
        );
    }

    #[test]
    fn a_reinstall_of_a_hook_that_is_not_installed_installs_nothing() {
        // Ruling S3: before the first install and after `uninstall` (exit,
        // or the session end) a late request installs no hook.
        let fake = FakeInstaller::new();
        assert_eq!(
            leaked().reinstall_keyboard_hook_with(&fake),
            Ok(Reinstalled::NotInstalled)
        );
        assert_eq!(fake.take_calls(), [], "never installed: no call");

        let hook = installed(&fake);
        assert_eq!(removed_keyboard_hook(hook, &fake), Some(1));
        assert_eq!(
            hook.reinstall_keyboard_hook_with(&fake),
            Ok(Reinstalled::NotInstalled)
        );
        assert_eq!(fake.take_calls(), [], "removed: no call");
        assert!(!hook.is_installed());
        assert_eq!(fake.keyboard_hooks(), [] as [usize; 0]);
    }

    #[test]
    fn a_reinstall_reports_an_old_hook_that_windows_removed_as_gone() {
        // KS-08: Windows removes a hook whose callback ran past
        // `LowLevelHooksTimeout`, so removing the old one fails; the new one
        // stays, and the failure is only reported.
        let fake = FakeInstaller::new();
        let hook = installed(&fake);
        fake.remove_on_its_own(1);

        assert_eq!(
            hook.reinstall_keyboard_hook_with(&fake),
            Ok(Reinstalled::Replaced {
                old: OldHook::Gone(ERROR_INVALID_HOOK_HANDLE.into())
            })
        );
        assert_eq!(
            fake.take_calls(),
            [Call::InstallKeyboard(4), Call::RemoveKeyboard(1)]
        );
        assert_eq!(fake.keyboard_hooks(), [4]);
        assert_eq!(removed_keyboard_hook(hook, &fake), Some(4));
    }

    #[test]
    fn the_daemons_reinstall_of_a_hook_never_installed_makes_no_call() {
        // The daemon's path, through the system installer, which panics at
        // any call under test: this returns only because the null handle
        // stops it first.
        assert_eq!(
            KeyboardHook::new().reinstall_keyboard_hook(),
            Ok(Reinstalled::NotInstalled)
        );
    }

    #[test]
    fn the_install_marks_the_hook_installed_and_the_removal_clears_it() {
        // Ruling I3: the watchdog counts raw input only while this flag
        // says the keyboard hook is installed.
        let fake = FakeInstaller::new();
        let hook = installed(&fake);
        assert!(
            hook.health().hook_installed(),
            "an installed hook is marked installed"
        );
        removed_keyboard_hook(hook, &fake);
        assert!(
            !hook.health().hook_installed(),
            "a removed hook is no longer marked installed"
        );
    }

    #[test]
    fn a_refused_install_leaves_the_hook_unmarked() {
        let fake = FakeInstaller::new();
        fake.refuse_next_keyboard_hook();
        let hook = leaked();
        assert!(hook.install_with(&fake).is_err(), "the install is refused");
        assert!(
            !hook.health().hook_installed(),
            "no hook, no mark: the watchdog counts no raw input"
        );
    }

    #[test]
    fn set_enabled_refuses_a_nested_call() {
        // A switch asked for while the main state is borrowed is refused and
        // changes nothing. On its own thread, so a broken refusal fails the
        // test after the timeout instead of the suite.
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _main = MainThread::new().expect("the main thread is set up");
            let hook = KeyboardHook::new();
            let nested = with_main(|_| with_main(|main| main.set_enabled(&hook, false)));
            let _ = tx.send((nested, hook.is_enabled()));
        });
        let (nested, enabled) = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("set_enabled panicked or hung inside with_main");
        assert_eq!(nested, Some(None));
        assert!(enabled, "a refused call leaves remapping on");
    }
}
