#[cfg(test)]
mod tests {
    use windows::Win32::System::Threading::{GetCurrentThread, GetThreadPriority};

    use crate::platform::priority::RaisedPriority;

    /// The calling thread's priority, relative to its process's class.
    fn priority() -> i32 {
        // SAFETY: the pseudo handle of the calling thread needs no closing,
        // and the call only reads that thread's priority.
        unsafe { GetThreadPriority(GetCurrentThread()) }
    }

    #[test]
    fn raise_sets_highest_and_drop_restores_normal() {
        // On a scratch thread, so that the test thread keeps its priority
        // whatever happens.
        let (inside, after) = std::thread::spawn(|| {
            let raised = RaisedPriority::raise().expect("a thread may raise its own priority");
            let inside = priority();
            drop(raised);
            (inside, priority())
        })
        .join()
        .expect("the scratch thread does not panic");
        assert_eq!(inside, 2, "THREAD_PRIORITY_HIGHEST while raised");
        assert_eq!(after, 0, "THREAD_PRIORITY_NORMAL once dropped");
    }
}
