#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::super::{TimerRequest, TimerSlot, TimerToken};
    use crate::engine::{Scheduler, TimerQueue};

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn request(seq: u64, due: Instant) -> TimerRequest {
        TimerRequest {
            token: TimerToken {
                generation: 0,
                slot: TimerSlot::Gesture(0x7C),
                seq,
            },
            due,
        }
    }

    /// The `seq` of every timer due by `now`, in the order the queue hands
    /// them out.
    fn due_order(queue: &mut TimerQueue, now: Instant) -> Vec<u64> {
        std::iter::from_fn(|| queue.take_due(now))
            .map(|request| request.token.seq)
            .collect()
    }

    #[test]
    fn the_queue_hands_out_due_timers_earliest_first_and_ties_in_start_order() {
        let t0 = Instant::now();
        let mut queue = TimerQueue::new();
        for (seq, due) in [(1, 30), (2, 10), (3, 30), (4, 20)] {
            assert!(
                queue.start(request(seq, t0 + ms(due))),
                "a queue never refuses"
            );
        }
        assert_eq!(queue.len(), 4);
        assert_eq!(queue.next_due(), Some(t0 + ms(10)));
        assert_eq!(queue.take_due(t0 + ms(9)), None, "nothing is due yet");

        assert_eq!(due_order(&mut queue, t0 + ms(20)), [2, 4]);
        assert_eq!(queue.next_due(), Some(t0 + ms(30)));
        assert_eq!(
            due_order(&mut queue, t0 + ms(30)),
            [1, 3],
            "equal deadlines come out in start order"
        );
        assert!(queue.is_empty());
        assert_eq!(queue.next_due(), None);
    }

    #[test]
    fn retain_keeps_only_live_timers() {
        let t0 = Instant::now();
        let mut queue = TimerQueue::new();
        for seq in 1..=3 {
            queue.start(request(seq, t0 + ms(10 * seq)));
        }

        queue.retain(&|token| token.seq != 2);
        assert_eq!(queue.len(), 2);
        assert_eq!(due_order(&mut queue, t0 + ms(100)), [1, 3]);

        queue.start(request(4, t0));
        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.take_due(t0 + ms(100)), None);
    }
}
