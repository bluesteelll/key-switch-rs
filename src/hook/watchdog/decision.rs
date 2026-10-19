//! The watchdog's decision (roadmap item 2.4, cut D3): counts and instants
//! in, "reinstall" or not out. No Win32 call, thread or clock: every instant
//! is an argument, so the fake-clock tests below drive it as the watchdog
//! thread does.
//!
//! While raw key events arrive, the watchdog ticks every [`TICK`] and reads
//! the keyboard callback's heartbeat. A raw key event that no callback
//! explains is evidence, counted only once it is at least one tick old: the
//! evidence is checked before the interval's raw events are added, so raw
//! input that arrives just before its own hook call never counts. The
//! interval's raw events are those its drains read: the drain of the
//! tick's own pass, right before the heartbeat is read, and those of any
//! earlier wake in the interval (ruling L1). A raw event read at the tick
//! `T` arrived at or before `T`, and is first checked at `T + TICK`.
//!
//! The counts are matched as totals, whichever of the two Windows delivered
//! first: the callbacks counted that no raw event has matched yet are a
//! credit for the raw events read later (ruling V3-I1). The credit is kept
//! while a drain leaves raw input in the queue, so a burst whose raw input
//! is read over several ticks is matched by the callbacks counted before
//! it, and a live hook is never reinstalled for it. It lapses at the first
//! tick whose drain emptied the queue: by then every raw event of those
//! callbacks has been read, and what is left is keys that raw input never
//! saw (keys the daemon swallows), which must not hide a hook that dies
//! later. A tick that sees a callback re-takes the baseline: the evidence
//! goes, so raw events that the hook legitimately does not see (another
//! desktop, UIPI) never add up across a live hook's ticks.
//!
//! # The anti-loop rule
//!
//! 1. A request needs at least 2 unexplained raw key events and no callback
//!    for `S = SILENCE * 2^min(n, MAX_BACKOFF_SHIFT)`, where `n` counts the
//!    reinstalls and failed attempts since the last callback seen. The
//!    silence is measured from the last tick that saw a callback, the last
//!    answer or decline, or the first raw event after idle, whichever came
//!    last: so keys arrived for all of `S` without a callback.
//! 2. At most one request is outstanding. It waits in the main thread's
//!    queue while that thread is busy, and no new evidence counts until the
//!    main thread answers.
//! 3. The main thread dispatches the hook calls that wait for it, then
//!    reinstalls only if the heartbeat has not moved since the request's
//!    snapshot (`main_thread::reinstall_if_silent`).
//! 4. After an answer, a new request needs a new window `S` and new
//!    evidence after the answer.
//! 5. Each reinstall or failed attempt with no callback in between doubles
//!    `S`, up to 64 s; the first callback seen resets it to 1 s.
//!
//! So a busy main thread draws at most one request per busy period, which
//! becomes a reinstall only if no hook call reaches the thread once it is
//! free: the thread was then unresponsive for at least `S`, at least the
//! largest `LowLevelHooksTimeout` Windows allows, while keys arrived, and
//! Windows removed the hook. A reinstall that does not help is spaced out.
//!
//! Ticking stops once no raw event came for [`ACTIVE_FOR`], nothing is
//! outstanding and no evidence waits for its window (ruling M1): idle costs
//! no wake-up.

#![forbid(unsafe_code)]

use std::time::{Duration, Instant};

use crate::hook::health::{ReinstallOutcome, ReinstallRequest};

/// Roadmap item 2.4: keys arrive and the callback stays silent "for more
/// than 1 s", the largest `LowLevelHooksTimeout` Windows allows.
pub(crate) const SILENCE: Duration = Duration::from_secs(1);
/// How often the heartbeat is read while keys are typed.
pub(crate) const TICK: Duration = Duration::from_millis(250);
/// How long ticking goes on after the last raw key event.
pub(crate) const ACTIVE_FOR: Duration = Duration::from_secs(2);
/// The silence a request needs doubles up to `SILENCE * 2^6`, 64 s.
pub(crate) const MAX_BACKOFF_SHIFT: u32 = 6;
/// How many unexplained raw key events a request needs.
const MIN_EVIDENCE: u64 = 2;

/// What a tick decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Ticking stopped: nothing to read until the next raw key event.
    Idle,
    /// Tick again at this instant.
    TickAt(Instant),
    /// Ask the main thread for `request`, and tick again at `next_tick`.
    Reinstall {
        request: ReinstallRequest,
        next_tick: Instant,
    },
}

/// The watchdog's state: counts and instants only, no key data.
#[derive(Debug)]
pub(crate) struct Watchdog {
    /// The heartbeat at the last tick.
    last_callbacks: u64,
    /// Every raw key event counted, and their count at the last tick.
    raw_events: u64,
    raw_at_last_tick: u64,
    /// The callbacks counted that no raw event has matched yet. They explain
    /// the raw events read next; what is left lapses at a tick whose drain
    /// emptied the queue.
    credit: u64,
    /// Raw key events that no callback explained, each at least a tick old.
    evidence: u64,
    /// Where the silence that a request needs starts (rule 1).
    quiet_since: Instant,
    /// The drain that read the newest raw key event, at most a tick after
    /// the event itself.
    last_raw_at: Option<Instant>,
    /// The next tick while ticking: the one source the watchdog thread sets
    /// its timer from (ruling I1).
    next_tick: Option<Instant>,
    /// A request waits for the main thread's answer.
    outstanding: bool,
    /// Reinstalls and failed attempts since the last callback seen.
    attempts: u32,
}

impl Watchdog {
    /// A watchdog that has seen `callbacks` heartbeats and no raw key event.
    pub(crate) fn new(now: Instant, callbacks: u64) -> Self {
        Self {
            last_callbacks: callbacks,
            raw_events: 0,
            raw_at_last_tick: 0,
            credit: 0,
            evidence: 0,
            quiet_since: now,
            last_raw_at: None,
            next_tick: None,
            outstanding: false,
            attempts: 0,
        }
    }

    /// `count` raw key events that the keyboard hook should see were read
    /// at `now`, by one drain. The first batch after idle starts the ticking
    /// and the silence (rule 1).
    pub(crate) fn raw_batch(&mut self, count: u64, now: Instant) {
        if count == 0 {
            return;
        }
        self.raw_events = self.raw_events.saturating_add(count);
        self.last_raw_at = Some(now);
        if self.next_tick.is_none() {
            self.next_tick = Some(now + TICK);
            self.quiet_since = now;
            self.credit = 0;
        }
    }

    /// One raw key event read at `now`: the decision tests' scripts read
    /// their events one by one.
    #[cfg(test)]
    pub(crate) fn raw_event(&mut self, now: Instant) {
        self.raw_batch(1, now);
    }

    /// When the next tick is due, while ticking.
    pub(crate) fn next_tick(&self) -> Option<Instant> {
        self.next_tick
    }

    /// The silence a request needs now (rule 5).
    fn window(&self) -> Duration {
        SILENCE * (1 << self.attempts.min(MAX_BACKOFF_SHIFT))
    }

    /// Reads the heartbeat, `callbacks`, at a tick that is due by `now`;
    /// `backlog` says that the drain right before it left raw input in the
    /// queue.
    pub(crate) fn tick(&mut self, now: Instant, callbacks: u64, backlog: bool) -> Verdict {
        let answered = callbacks.wrapping_sub(self.last_callbacks);
        let arrived = self.raw_events.saturating_sub(self.raw_at_last_tick);
        self.last_callbacks = callbacks;
        self.raw_at_last_tick = self.raw_events;

        // The callbacks counted before explain this interval's raw events
        // first; the rest of their credit waits only for raw input still
        // queued.
        let explained = arrived.min(self.credit);
        let unexplained = arrived - explained;
        let kept = if backlog { self.credit - explained } else { 0 };

        let mut request = None;
        if answered > 0 {
            // The baseline: the callback is alive. Raw events no callback
            // explains are dropped; this interval's callbacks beyond them are
            // credit for raw events read later.
            self.quiet_since = now;
            self.evidence = 0;
            self.credit = kept.saturating_add(answered.saturating_sub(unexplained));
            self.attempts = 0;
        } else {
            self.credit = kept;
            // Before this interval's raw events count (ruling M5).
            let decide = !self.outstanding
                && self.evidence >= MIN_EVIDENCE
                && now >= self.quiet_since + self.window();
            self.evidence = self.evidence.saturating_add(unexplained);
            if decide {
                self.outstanding = true;
                request = Some(ReinstallRequest {
                    seen: callbacks,
                    silent_for: now.saturating_duration_since(self.quiet_since),
                    attempt: self.attempts.saturating_add(1),
                });
            }
        }

        let next_tick = now + TICK;
        if let Some(request) = request {
            self.next_tick = Some(next_tick);
            return Verdict::Reinstall { request, next_tick };
        }
        if self.keeps_ticking(now) {
            self.next_tick = Some(next_tick);
            Verdict::TickAt(next_tick)
        } else {
            self.next_tick = None;
            Verdict::Idle
        }
    }

    /// Whether another tick can change anything (ruling M1): a request
    /// waits, a raw event came within [`ACTIVE_FOR`], or evidence that can
    /// make a request waits for its window.
    fn keeps_ticking(&self, now: Instant) -> bool {
        let Some(last_raw_at) = self.last_raw_at else {
            return self.outstanding;
        };
        self.outstanding
            || now < last_raw_at + ACTIVE_FOR
            || (self.evidence >= MIN_EVIDENCE && now < last_raw_at + self.window() + TICK)
    }

    /// The main thread answered the outstanding request with `outcome`.
    /// A reinstall or a failed attempt doubles the next window; every answer
    /// starts a new window and drops the evidence (rule 4), and the raw
    /// events before the answer no longer count. The heartbeat's base stays,
    /// so a callback of the new hook resets the window at the next tick.
    pub(crate) fn answered(&mut self, now: Instant, outcome: ReinstallOutcome) {
        self.outstanding = false;
        match outcome {
            ReinstallOutcome::Reinstalled | ReinstallOutcome::Failed => {
                self.attempts = self.attempts.saturating_add(1);
            }
            ReinstallOutcome::Alive | ReinstallOutcome::NotInstalled => {}
        }
        self.start_over(now);
    }

    /// The request could not be sent (the main thread's queue was full, or
    /// it is ending): the next tick asks again.
    pub(crate) fn not_sent(&mut self) {
        self.outstanding = false;
    }

    /// Keyboard input is not on the daemon's desktop at the decision: the
    /// evidence is dropped, and a new request needs a new window.
    pub(crate) fn declined(&mut self, now: Instant) {
        self.outstanding = false;
        self.start_over(now);
    }

    fn start_over(&mut self, now: Instant) {
        self.evidence = 0;
        self.credit = 0;
        self.quiet_since = now;
        self.raw_at_last_tick = self.raw_events;
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::hook::health::{ReinstallOutcome, ReinstallRequest};

    /// A fake clock in whole milliseconds that drives a [`Watchdog`] the way
    /// the watchdog thread does: at each millisecond the script runs first
    /// (raw key events, callbacks, answers), then the tick, if the watchdog's
    /// next tick is due.
    struct Feeder {
        watchdog: Watchdog,
        start: Instant,
        /// The heartbeat: every callback the script made.
        callbacks: u64,
        /// When each request was made, in ms, and the request.
        requests: Vec<(u64, ReinstallRequest)>,
        /// When each tick ran, in ms.
        ticks: Vec<u64>,
        /// When a tick stopped the ticking, in ms.
        idle: Vec<u64>,
        /// Whether the drain before the next tick leaves raw input queued.
        backlog: bool,
    }

    impl Feeder {
        fn new() -> Self {
            let start = Instant::now();
            Self {
                watchdog: Watchdog::new(start, 0),
                start,
                callbacks: 0,
                requests: Vec::new(),
                ticks: Vec::new(),
                idle: Vec::new(),
                backlog: false,
            }
        }

        fn at(&self, ms: u64) -> Instant {
            self.start + Duration::from_millis(ms)
        }

        fn raw(&mut self, ms: u64) {
            let now = self.at(ms);
            self.watchdog.raw_event(now);
        }

        /// `count` raw key events read at `ms`.
        fn raws(&mut self, ms: u64, count: u64) {
            for _ in 0..count {
                self.raw(ms);
            }
        }

        fn callback(&mut self) {
            self.callbacks += 1;
        }

        fn answer(&mut self, ms: u64, outcome: ReinstallOutcome) {
            let now = self.at(ms);
            self.watchdog.answered(now, outcome);
        }

        /// The time of the last request, if any.
        fn last_request(&self) -> Option<u64> {
            self.requests.last().map(|(at, _)| *at)
        }

        fn request_times(&self) -> Vec<u64> {
            self.requests.iter().map(|(at, _)| *at).collect()
        }

        /// Runs the clock from `from` to `to` ms, both included.
        fn run(&mut self, from: u64, to: u64, mut script: impl FnMut(u64, &mut Self)) {
            for ms in from..=to {
                script(ms, self);
                let now = self.at(ms);
                if self.watchdog.next_tick().is_some_and(|due| due <= now) {
                    self.ticks.push(ms);
                    match self.watchdog.tick(now, self.callbacks, self.backlog) {
                        Verdict::Reinstall { request, next_tick } => {
                            assert_eq!(next_tick, now + TICK, "a request ticks on");
                            self.requests.push((ms, request));
                        }
                        Verdict::Idle => self.idle.push(ms),
                        Verdict::TickAt(at) => assert_eq!(at, now + TICK),
                    }
                    assert_eq!(
                        self.watchdog.next_tick(),
                        match self.idle.last() {
                            Some(&idle) if idle == ms => None,
                            _ => Some(now + TICK),
                        },
                        "the next tick is the one source of the timer (ruling I1)"
                    );
                }
            }
        }
    }

    /// A raw key event every 30 ms, from 0.
    fn typing(ms: u64, f: &mut Feeder) {
        if ms.is_multiple_of(30) {
            f.raw(ms);
        }
    }

    #[test]
    fn key_events_with_a_silent_callback_for_more_than_1_s_ask_for_exactly_one_reinstall() {
        // Roadmap item 2.4's acceptance: keys arrive and the callback stays
        // silent. The request stays outstanding while nobody answers, so
        // there is exactly one.
        let mut f = Feeder::new();
        f.run(0, 5000, typing);
        assert_eq!(f.requests.len(), 1, "{:?}", f.requests);
        let (at, request) = f.requests[0];
        assert!((1000..=1500).contains(&at), "requested at {at} ms");
        assert_eq!(request.seen, 0, "the heartbeat the decision read");
        assert!(request.silent_for >= SILENCE, "{request:?}");
        assert_eq!(request.attempt, 1);
    }

    #[test]
    fn a_reinstall_that_brings_the_callbacks_back_asks_for_nothing_more() {
        let mut f = Feeder::new();
        let mut answered = false;
        f.run(0, 10_000, |ms, f| {
            typing(ms, f);
            if answered && ms % 30 == 1 {
                f.callback();
            }
            if f.last_request().is_some_and(|at| ms == at + 50) {
                f.answer(ms, ReinstallOutcome::Reinstalled);
                answered = true;
            }
        });
        assert_eq!(f.requests.len(), 1, "{:?}", f.requests);
    }

    #[test]
    fn a_callback_that_keeps_up_asks_for_nothing() {
        // Whatever the order of raw input and the hook call: the callback 1
        // ms before each raw event, 1 ms after it, and callbacks late in each
        // tick interval whose raw events land early in the next.
        let mut before = Feeder::new();
        before.run(0, 10_000, |ms, f| match ms % 30 {
            0 => f.callback(),
            1 => f.raw(ms),
            _ => {}
        });
        assert_eq!(before.requests, [], "callback before its raw event");

        let mut after = Feeder::new();
        after.run(0, 10_000, |ms, f| match ms % 30 {
            0 => f.raw(ms),
            1 => f.callback(),
            _ => {}
        });
        assert_eq!(after.requests, [], "callback after its raw event");

        let mut across = Feeder::new();
        across.run(0, 10_000, |ms, f| {
            if ms == 0 {
                f.raw(ms);
                f.callback();
            }
            match ms % 250 {
                230..=249 => f.callback(),
                1..=20 if ms > 250 => f.raw(ms),
                _ => {}
            }
        });
        assert!(across.ticks.len() > 30, "it ticked: {:?}", across.ticks);
        assert_eq!(across.requests, [], "raw events across the tick");
    }

    #[test]
    fn idle_asks_for_nothing_and_stops_ticking() {
        let mut f = Feeder::new();
        f.run(0, 600_000, |ms, f| {
            if ms == 0 {
                f.raw(ms);
                f.callback();
            }
        });
        assert_eq!(f.requests, []);
        assert_eq!(f.idle.len(), 1, "{:?}", f.idle);
        assert!(f.idle[0] <= 2250, "idle at {} ms", f.idle[0]);
        assert_eq!(f.ticks.last(), f.idle.last(), "no tick after the idle one");
    }

    #[test]
    fn a_busy_main_thread_gives_no_reinstall_loop() {
        // The request stays outstanding while the main thread is busy; once
        // it is free the hook calls that waited are dispatched (+50) and the
        // hook answers.
        let mut f = Feeder::new();
        f.run(0, 20_000, |ms, f| {
            typing(ms, f);
            if ms == 10_000 {
                f.callbacks += 50;
                f.answer(ms, ReinstallOutcome::Alive);
            }
            if ms > 10_000 && ms % 30 == 1 {
                f.callback();
            }
        });
        assert_eq!(f.requests.len(), 1, "{:?}", f.request_times());
    }

    /// Silent typing for `until` ms; every request is answered `Reinstalled`
    /// 10 ms after it.
    fn reinstalls_without_callbacks(until: u64) -> Feeder {
        let mut f = Feeder::new();
        f.run(0, until, |ms, f| {
            typing(ms, f);
            if f.last_request().is_some_and(|at| ms == at + 10) {
                f.answer(ms, ReinstallOutcome::Reinstalled);
            }
        });
        f
    }

    #[test]
    fn reinstalls_that_bring_no_callback_back_are_spaced_out() {
        // The anti-loop rule's backoff: the silence a request needs doubles
        // with each attempt, 1, 2, 4 ... 64 s, and stays at 64 s.
        let f = reinstalls_without_callbacks(250_000);
        let times = f.request_times();
        assert_eq!(times.len(), 8, "{times:?}");
        assert!((1000..=1500).contains(&times[0]), "{times:?}");
        for (index, gap) in times.windows(2).map(|pair| pair[1] - pair[0]).enumerate() {
            let window = 1000 * (1 << (index + 1).min(MAX_BACKOFF_SHIFT as usize)) as u64;
            assert!(
                (window..window + 500).contains(&gap),
                "gap {index}: {gap} ms, window {window} ms: {times:?}"
            );
        }
        let attempts: Vec<u32> = f.requests.iter().map(|(_, r)| r.attempt).collect();
        assert_eq!(attempts, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn failed_reinstalls_are_spaced_out_like_reinstalls() {
        // A reinstall that Windows refused is an attempt too: asking again
        // after 1 s each time would be the loop the anti-loop rule forbids.
        let mut f = Feeder::new();
        f.run(0, 20_000, |ms, f| {
            typing(ms, f);
            if f.last_request().is_some_and(|at| ms == at + 10) {
                f.answer(ms, ReinstallOutcome::Failed);
            }
        });
        let times = f.request_times();
        assert_eq!(times.len(), 4, "{times:?}");
        for (index, gap) in times.windows(2).map(|pair| pair[1] - pair[0]).enumerate() {
            let window = 1000 * (1 << (index + 1)) as u64;
            assert!(
                (window..window + 500).contains(&gap),
                "gap {index}: {gap} ms, window {window} ms: {times:?}"
            );
        }
        let attempts: Vec<u32> = f.requests.iter().map(|(_, r)| r.attempt).collect();
        assert_eq!(attempts, [1, 2, 3, 4], "each failed attempt counts");
    }

    #[test]
    fn the_silence_resets_to_1_s_once_a_callback_is_seen() {
        let mut f = Feeder::new();
        let mut callback_tick = None;
        f.run(0, 20_000, |ms, f| {
            typing(ms, f);
            if f.requests.len() < 4 && f.last_request().is_some_and(|at| ms == at + 10) {
                f.answer(ms, ReinstallOutcome::Reinstalled);
            }
            if f.requests.len() == 3 && callback_tick.is_none() && ms % 250 == 100 {
                f.callback();
                callback_tick = Some(ms + 150);
            }
        });
        let times = f.request_times();
        assert!(times.len() >= 4, "{times:?}");
        let tick = callback_tick.expect("a callback came after the third request");
        let next = times[3] - tick;
        assert!(
            (1000..2000).contains(&next),
            "the request after the callback came {next} ms after its tick: {times:?}"
        );
    }

    #[test]
    fn a_single_raw_event_without_a_callback_asks_for_nothing() {
        // Ruling M1: one unexplained raw event is no evidence, and it does
        // not keep the watchdog ticking.
        let mut f = Feeder::new();
        f.run(0, 10_000, |ms, f| {
            if ms == 0 {
                f.raw(ms);
            }
        });
        assert_eq!(f.requests, []);
        assert_eq!(f.idle.len(), 1, "{:?}", f.idle);
        assert!(f.idle[0] <= 2250, "idle at {} ms", f.idle[0]);
        assert_eq!(f.ticks.last(), f.idle.last(), "no tick after the idle one");
    }

    #[test]
    fn raw_events_of_callbacks_seen_at_the_last_tick_are_explained() {
        // The credit: callbacks counted at a tick explain the raw events of
        // the next interval.
        let mut f = Feeder::new();
        f.run(0, 10_000, |ms, f| {
            if ms == 0 {
                f.raw(ms);
                f.callback();
            }
            if (230..=249).contains(&ms) {
                f.callback();
            }
            if (251..=270).contains(&ms) {
                f.raw(ms);
            }
        });
        assert_eq!(f.requests, []);
    }

    #[test]
    fn evidence_is_checked_before_this_intervals_raw_events_are_added() {
        // Ruling M5: after a second of quiet, a key down and up whose raw
        // events land just before a tick and whose callbacks land just after
        // it is no evidence yet, and the next tick sees the callbacks.
        let mut f = Feeder::new();
        f.run(0, 5000, |ms, f| {
            if ms == 0 {
                f.raw(ms);
                f.callback();
            }
            if ms == 1248 || ms == 1249 {
                f.raw(ms);
            }
            if ms == 1251 || ms == 1252 {
                f.callback();
            }
        });
        assert!(f.ticks.contains(&1250), "a tick at 1250 ms: {:?}", f.ticks);
        assert_eq!(f.requests, []);
    }

    #[test]
    fn a_key_after_idle_waits_a_full_second_of_silence() {
        // The silence that a request needs starts no earlier than the first
        // raw event after idle: a main thread busy for 600 ms when typing
        // resumes, whose hook calls are then dispatched, draws no request.
        let mut f = Feeder::new();
        f.run(0, 8000, |ms, f| {
            if ms == 0 {
                f.raw(ms);
                f.callback();
            }
            if ms >= 5000 && ms % 30 == 20 {
                f.raw(ms);
            }
            // The 20 hook calls of 5000-5570 ms, dispatched at once.
            if ms == 5600 {
                f.callbacks += 20;
            }
            if ms > 5600 && ms % 30 == 21 {
                f.callback();
            }
        });
        assert!(
            f.idle.first().is_some_and(|&idle| idle < 5000),
            "idle before the typing: {:?}",
            f.idle
        );
        assert_eq!(f.requests, []);
    }

    #[test]
    fn evidence_keeps_the_watchdog_ticking_until_its_window_and_then_stops() {
        // Ruling M1: after two reinstalls the window is 4 s, longer than the
        // 2 s that ticking lasts after the last raw event. Evidence keeps it
        // ticking until its window has passed, then ticking stops.
        let mut f = Feeder::new();
        f.run(0, 20_000, |ms, f| {
            if ms <= 3300 {
                typing(ms, f);
            }
            if f.last_request().is_some_and(|at| ms == at + 10) {
                f.answer(ms, ReinstallOutcome::Reinstalled);
            }
        });
        let times = f.request_times();
        assert_eq!(times.len(), 3, "{times:?}");
        assert!(times[2] > 3300 + 2000, "{times:?}");
        assert_eq!(f.idle.len(), 1, "{:?}", f.idle);
        assert!(
            f.idle[0] <= times[2] + 500,
            "idle at {} ms: {times:?}",
            f.idle[0]
        );
        assert_eq!(f.ticks.last(), f.idle.last(), "no tick after the idle one");
    }

    #[test]
    fn raw_events_before_an_answer_are_no_evidence_after_it() {
        // Rule 4: a new request needs evidence after the answer. Two raw
        // events between the request's tick and its answer, then nothing.
        let mut f = Feeder::new();
        f.run(0, 10_000, |ms, f| {
            if ms <= 990 {
                typing(ms, f);
            }
            if ms == 1001 || ms == 1003 {
                f.raw(ms);
            }
            if f.last_request().is_some_and(|at| ms == at + 10) {
                f.answer(ms, ReinstallOutcome::Reinstalled);
            }
        });
        assert_eq!(f.request_times(), [1000]);
    }

    #[test]
    fn a_burst_read_over_several_ticks_is_matched_by_the_callbacks_counted_before_it() {
        // Ruling V3-I1, the critic's trace: an injected burst of 10 000 key
        // events reaches the hook at once, and its raw input is read over
        // three ticks because each drain ran out of its time budget. The
        // callbacks counted at the first tick explain every raw event read
        // after it, so a second of silence after the burst draws nothing.
        let mut f = Feeder::new();
        f.run(0, 5000, |ms, f| match ms {
            0 => {
                f.raw(ms);
                f.callback();
            }
            100 => f.callbacks += 10_000,
            249 => {
                f.raws(ms, 1_000);
                f.backlog = true;
            }
            499 => f.raws(ms, 4_864),
            749 => {
                f.raws(ms, 4_136);
                f.backlog = false;
            }
            _ => {}
        });
        assert!(f.ticks.contains(&750), "{:?}", f.ticks);
        assert_eq!(f.request_times(), [] as [u64; 0]);
    }

    #[test]
    fn a_hook_that_dies_during_a_burst_asks_for_exactly_one_reinstall() {
        // Ruling V3-I1: Windows removed the hook after 5 000 of the burst's
        // 10 000 events. Raw events beyond the callbacks counted are
        // evidence, also when they are read at a later tick.
        let mut f = Feeder::new();
        f.run(0, 5000, |ms, f| match ms {
            0 => {
                f.raw(ms);
                f.callback();
            }
            100 => f.callbacks += 5_000,
            249 => {
                f.raws(ms, 6_000);
                f.backlog = true;
            }
            499 => {
                f.raws(ms, 4_000);
                f.backlog = false;
            }
            _ => {}
        });
        let times = f.request_times();
        assert_eq!(times.len(), 1, "{times:?}");
        assert!((1250..=1500).contains(&times[0]), "{times:?}");
    }

    #[test]
    fn a_hook_that_dies_after_a_burst_asks_for_exactly_one_reinstall() {
        // Ruling V3-I1: the burst held 500 events that the hook saw and
        // raw input did not (keys the daemon swallowed), then the hook died
        // and typing went on. Callbacks that no raw event matched are no
        // credit once a drain has emptied the queue, so the silence counts
        // from the burst's tick.
        let mut f = Feeder::new();
        f.run(0, 10_000, |ms, f| {
            match ms {
                0 => {
                    f.raw(ms);
                    f.callback();
                }
                100 => f.callbacks += 10_500,
                249 => f.raws(ms, 10_000),
                _ => {}
            }
            if ms >= 300 {
                typing(ms, f);
            }
        });
        let times = f.request_times();
        assert_eq!(times.len(), 1, "{times:?}");
        assert!((1250..=1500).contains(&times[0]), "{times:?}");
    }

    /// Ten minutes of typing every 30 ms, with the hook alive and the
    /// counts apart by `drift`, which adds raw events or callbacks.
    fn ten_minutes_of_drift(drift: impl Fn(u64, &mut Feeder)) -> Feeder {
        let mut f = Feeder::new();
        f.run(0, 600_000, |ms, f| {
            typing(ms, f);
            drift(ms, f);
        });
        f
    }

    #[test]
    fn long_drift_with_a_live_hook_asks_for_nothing() {
        // Ruling V3-I1: raw events the hook legitimately does not see
        // (another desktop, UIPI) and callbacks raw input does not see
        // (keys the daemon swallows) never add up to a request: a tick that
        // sees the callback alive drops the evidence, and the credit that
        // a drain has settled.
        let more_raw = ten_minutes_of_drift(|ms, f| {
            if ms % 30 == 1 {
                f.callback();
            }
            if ms % 250 == 125 {
                f.raw(ms);
            }
        });
        assert_eq!(
            more_raw.request_times(),
            [] as [u64; 0],
            "raw input sees more"
        );
        let more_callbacks = ten_minutes_of_drift(|ms, f| {
            if ms % 30 == 1 || ms % 100 == 50 {
                f.callback();
            }
        });
        assert_eq!(
            more_callbacks.request_times(),
            [] as [u64; 0],
            "the hook sees more"
        );
        let one_in_eight = ten_minutes_of_drift(|ms, f| {
            if ms % 240 == 1 {
                f.callback();
            }
        });
        assert_eq!(
            one_in_eight.request_times(),
            [] as [u64; 0],
            "the hook sees one key in eight"
        );
    }

    #[test]
    fn a_hook_that_dies_after_long_drift_is_found_within_its_window() {
        // Ruling V3-I1: ten minutes in which the hook saw more than raw
        // input leave no credit that hides its death.
        let mut f = Feeder::new();
        let mut last_callback = 0;
        f.run(0, 610_000, |ms, f| {
            typing(ms, f);
            if ms < 600_000 && (ms % 30 == 1 || ms % 100 == 50) {
                f.callback();
                last_callback = ms;
            }
        });
        let times = f.request_times();
        assert_eq!(times.len(), 1, "{times:?}");
        assert!(
            (last_callback + 1000..=last_callback + 1500).contains(&times[0]),
            "the last callback at {last_callback} ms: {times:?}"
        );
    }

    #[test]
    fn a_batch_counts_as_that_many_raw_events() {
        // Ruling L1: a drain hands the decision its count in one call, which
        // is the same as one call per raw event at the drain's instant.
        let start = Instant::now();
        let at = |ms: u64| start + Duration::from_millis(ms);
        for count in [1, 2, 7, 250] {
            let mut batched = Watchdog::new(start, 0);
            let mut single = Watchdog::new(start, 0);
            batched.raw_batch(count, at(10));
            for _ in 0..count {
                single.raw_event(at(10));
            }
            assert_eq!(format!("{batched:?}"), format!("{single:?}"), "{count}");
            for ms in [260, 510] {
                assert_eq!(
                    batched.tick(at(ms), 0, false),
                    single.tick(at(ms), 0, false),
                    "{count} raw events, the tick at {ms} ms"
                );
            }
            assert_eq!(format!("{batched:?}"), format!("{single:?}"), "{count}");
        }
        let mut idle = Watchdog::new(start, 0);
        let before = format!("{idle:?}");
        idle.raw_batch(0, at(10));
        assert_eq!(
            format!("{idle:?}"),
            before,
            "an empty batch changes nothing"
        );
    }

    #[test]
    fn a_declined_request_needs_fresh_evidence() {
        // Keyboard input is not on this desktop at the decision: the
        // evidence is dropped, and a new request needs a new window.
        let mut f = Feeder::new();
        let mut declined_at = None;
        f.run(0, 5000, |ms, f| {
            typing(ms, f);
            if declined_at.is_none() && f.last_request() == Some(ms.saturating_sub(1)) && ms > 0 {
                let now = f.at(ms);
                f.watchdog.declined(now);
                declined_at = Some(ms);
            }
        });
        let times = f.request_times();
        let declined = declined_at.expect("the first request was declined");
        assert!(times.len() >= 2, "{times:?}");
        assert!(
            times[1] >= declined + 1000,
            "asked again {} ms after the decline: {times:?}",
            times[1] - declined
        );
    }

    #[test]
    fn a_request_that_could_not_be_sent_is_asked_again_at_the_next_tick() {
        let mut f = Feeder::new();
        let mut not_sent = false;
        f.run(0, 3000, |ms, f| {
            typing(ms, f);
            if !not_sent && f.last_request().is_some_and(|at| ms == at + 1) {
                f.watchdog.not_sent();
                not_sent = true;
            }
        });
        let times = f.request_times();
        assert!(times.len() >= 2, "{times:?}");
        assert_eq!(times[1], times[0] + 250, "{times:?}");
    }
}
