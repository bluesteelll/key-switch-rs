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
            }
        }

        fn at(&self, ms: u64) -> Instant {
            self.start + Duration::from_millis(ms)
        }

        fn raw(&mut self, ms: u64) {
            let now = self.at(ms);
            self.watchdog.raw_event(now);
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
                    match self.watchdog.tick(now, self.callbacks) {
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
        if ms % 30 == 0 {
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
        // The owed credit: callbacks counted at a tick explain the raw
        // events of the next interval.
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
