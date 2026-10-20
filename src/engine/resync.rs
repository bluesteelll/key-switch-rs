//! When the main thread reads the modifiers from Windows between key events
//! (roadmap item 2.5, cut D4 to D6).
//!
//! The keyboard callback no longer reads the modifier state at every event:
//! the engine follows Shift, Ctrl, Alt and Win from the events it sees. What
//! it cannot see is counted instead. Every source that can hide a key event
//! from the hook moves the hiding count: the daemon's start, a foreground
//! change, `swch on`, a hook reinstall, keyboard input leaving this desktop
//! and coming back, and the adapter's notes of hidden input (a refused or
//! late callback, a long stretch away from the messages, a gap the hook
//! watchdog saw), which [`Resync::plan`] folds in. The main thread's drain
//! asks for a plan between events, never inside a callback, and reads the
//! codes it names.
//!
//! [`Resync`] is pure data: it reads no clock and no key state.
//!
//! - A count that moved plans a read of every modifier code once.
//! - A code touched by an event since the last plan is left out of that
//!   read and owed to the next plan: its flag comes from its own latest
//!   event, and Windows may not have applied that event yet ("the callback
//!   function is called before the asynchronous state of the key is
//!   updated", Microsoft Learn, LowLevelKeyboardProc).
//! - Nothing is read while keyboard input is on another desktop, where the
//!   key state is no witness, until a signal that it came back or the next
//!   decided key event, which proves that it did.

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use super::modifiers::ModifierSet;

/// The hiding count and what the next read leaves out (roadmap item 2.5).
pub(super) struct Resync {
    /// Moves once per source that can have hidden a key event. Starts at 1,
    /// so the first plan reads (the daemon's start).
    hiding: u64,
    /// `hiding` at the last read of every code. Starts at 0.
    resynced: u64,
    /// The adapter's note count at the last plan.
    notes_seen: u64,
    /// Touched by an event since the last plan.
    fresh: ModifierSet,
    /// Left out of the last planned read because fresh then.
    owed: ModifierSet,
    /// Keyboard input left this desktop, and nothing proved it came back.
    input_away: bool,
}

impl Resync {
    pub(super) fn new() -> Self {
        Self {
            hiding: 1,
            resynced: 0,
            notes_seen: 0,
            fresh: ModifierSet::EMPTY,
            owed: ModifierSet::EMPTY,
            input_away: false,
        }
    }

    /// Something may have hidden a key event from the hook.
    pub(super) fn bump(&mut self) {
        self.hiding = self.hiding.wrapping_add(1);
    }

    /// A key event of `vk` changed a flag: decided by the engine, or the
    /// daemon's own injected event, which it only follows. A decided event
    /// proves that keyboard input is on this desktop; an injected one does
    /// not.
    pub(super) fn event(&mut self, vk: VIRTUAL_KEY, decided: bool) {
        self.fresh = self.fresh.union(ModifierSet::touched_by(vk));
        if decided {
            self.input_away = false;
        }
    }

    /// Keyboard input left this desktop: nothing is read until it is back.
    pub(super) fn input_left(&mut self) {
        self.input_away = true;
        self.bump();
    }

    /// Keyboard input came back to this desktop.
    pub(super) fn input_returned(&mut self) {
        self.input_away = false;
        self.bump();
    }

    /// Every key was read from Windows (a reset of `swch on` or a hook
    /// reinstall): nothing is due or owed.
    pub(super) fn read_everything(&mut self) {
        self.resynced = self.hiding;
        self.fresh = ModifierSet::EMPTY;
        self.owed = ModifierSet::EMPTY;
    }

    /// The codes the drain reads now, with `notes` the adapter's note count;
    /// empty if none is due. Reads no clock.
    pub(super) fn plan(&mut self, notes: u64) -> ModifierSet {
        if notes != self.notes_seen {
            self.notes_seen = notes;
            self.bump();
        }
        if self.input_away {
            return ModifierSet::EMPTY;
        }
        let planned = if self.hiding != self.resynced {
            self.resynced = self.hiding;
            ModifierSet::ALL
        } else {
            self.owed
        };
        let read = planned.without(self.fresh);
        self.owed = planned.without(read);
        self.fresh = ModifierSet::EMPTY;
        read
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;

    /// A resync whose start has been read, with nothing due.
    fn settled() -> Resync {
        let mut resync = Resync::new();
        assert_eq!(resync.plan(0), ModifierSet::ALL, "the start reads");
        resync
    }

    #[test]
    fn a_new_resync_plans_a_full_read_at_its_first_plan() {
        // E0: a modifier held when the daemon starts is read at its first
        // drain.
        let mut resync = Resync::new();
        assert_eq!(resync.plan(0), ModifierSet::ALL);
        assert_eq!(resync.plan(0), ModifierSet::EMPTY, "and only once");
    }

    #[test]
    fn a_bump_plans_one_full_read() {
        let mut resync = settled();
        resync.bump();
        assert_eq!(resync.plan(0), ModifierSet::ALL);
        assert_eq!(resync.plan(0), ModifierSet::EMPTY);
        resync.bump();
        resync.bump();
        assert_eq!(resync.plan(0), ModifierSet::ALL, "two bumps, one read");
        assert_eq!(resync.plan(0), ModifierSet::EMPTY);
    }

    #[test]
    fn a_changed_note_count_plans_a_full_read_a_smaller_one_included() {
        let mut resync = settled();
        assert_eq!(resync.plan(3), ModifierSet::ALL, "notes 0 to 3");
        assert_eq!(resync.plan(3), ModifierSet::EMPTY, "no change");
        assert_eq!(
            resync.plan(2),
            ModifierSet::ALL,
            "a smaller count is a change too (a sum that wrapped)"
        );
        assert_eq!(resync.plan(2), ModifierSet::EMPTY);
    }

    #[test]
    fn a_modifier_touched_since_the_last_plan_is_left_out_and_read_at_the_next() {
        let mut resync = settled();
        resync.event(VK_LSHIFT, true);
        resync.bump();
        let first = resync.plan(0);
        assert!(!first.contains(VK_LSHIFT), "fresh: its own event is newer");
        assert!(!first.contains(VK_SHIFT), "and its generic code");
        assert!(first.contains(VK_LCONTROL) && first.contains(VK_RSHIFT));
        assert_eq!(
            first.union(ModifierSet::touched_by(VK_LSHIFT)),
            ModifierSet::ALL
        );
        assert_eq!(
            resync.plan(0),
            ModifierSet::touched_by(VK_LSHIFT),
            "owed to the next plan"
        );
        assert_eq!(resync.plan(0), ModifierSet::EMPTY, "and then settled");

        resync.event(VK_LSHIFT, true);
        assert_eq!(
            resync.plan(0),
            ModifierSet::EMPTY,
            "a plan with nothing due owes nothing"
        );
        assert_eq!(resync.plan(0), ModifierSet::EMPTY);

        resync.bump();
        resync.event(VK_LCONTROL, true);
        let _ = resync.plan(0);
        resync.event(VK_LCONTROL, true);
        assert_eq!(
            resync.plan(0),
            ModifierSet::EMPTY,
            "an owed code touched again is left out again"
        );
        assert_eq!(
            resync.plan(0),
            ModifierSet::touched_by(VK_LCONTROL),
            "and still owed"
        );
    }

    #[test]
    fn nothing_is_read_while_input_is_away_until_it_returns_or_a_key_event_is_decided() {
        let mut resync = settled();
        resync.input_left();
        assert_eq!(resync.plan(0), ModifierSet::EMPTY, "away");
        assert_eq!(resync.plan(1), ModifierSet::EMPTY, "a note while away");
        resync.input_returned();
        assert_eq!(resync.plan(1), ModifierSet::ALL, "back");

        resync.input_left();
        assert_eq!(resync.plan(1), ModifierSet::EMPTY);
        resync.event(VK_F13, true);
        assert_eq!(
            resync.plan(1),
            ModifierSet::ALL,
            "a decided key event proves input is back"
        );
    }

    #[test]
    fn an_injected_event_marks_its_modifier_fresh_and_does_not_bring_input_back() {
        let mut resync = settled();
        resync.input_left();
        resync.event(VK_LCONTROL, false);
        assert_eq!(resync.plan(0), ModifierSet::EMPTY, "still away");
        resync.input_returned();
        resync.event(VK_LCONTROL, false);
        let read = resync.plan(0);
        assert!(!read.contains(VK_LCONTROL), "fresh");
        assert!(read.contains(VK_LSHIFT));
    }
}
