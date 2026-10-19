//! The `swch` client (`ipc::request`) against test pipes with unique names,
//! and the daemon's pipe server (`ipc_server::bind`, then
//! `IpcListener::serve`) driven through a stand-in for the daemon.
//!
//! No test here installs a keyboard hook or talks to a daemon: every pipe
//! is created by the test itself, under a name no daemon uses, and the
//! server gets a `DaemonControl` that only records what it is asked.

use std::io::{BufRead, BufReader, Write};
use std::num::NonZeroU8;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant, UNIX_EPOCH};

use interprocess::local_socket::{GenericNamespaced, Listener, ListenerOptions, prelude::*};
use interprocess::os::windows::ToWtf16;
use interprocess::os::windows::local_socket::ListenerOptionsExt;
use interprocess::os::windows::named_pipe::{DuplexPipeStream, PipeListenerOptions, pipe_mode};
use interprocess::os::windows::security_descriptor::SecurityDescriptor;

use key_switch_rs::hook::health::HookHealth;
use key_switch_rs::hook::ipc_server::{self, DaemonControl, DaemonStatus};
use key_switch_rs::ipc::{Command, REPLY_TIMEOUT, RequestError, pipe_path, request};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A pipe name no other test, test run or daemon uses.
fn unique_pipe_name(tag: &str) -> String {
    // Relaxed is enough: the counter only hands out distinct numbers and
    // publishes no other data.
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("key-switch-rs-test-{}-{tag}-{n}.sock", std::process::id())
}

fn listener_options(name: &str) -> ListenerOptions<'_> {
    ListenerOptions::new().name(
        name.to_ns_name::<GenericNamespaced>()
            .expect("a test pipe name is valid"),
    )
}

fn listener(name: &str) -> Listener {
    listener_options(name)
        .create_sync()
        .expect("the test pipe can be created")
}

/// A stand-in for the daemon: a remapping flag, two bindings, the hook's
/// health, and a record of every switch it was asked for.
struct TestControl {
    enabled: AtomicBool,
    bindings: usize,
    hook: HookHealth,
    toggles: Mutex<Vec<bool>>,
}

/// A watched hook that was never reinstalled.
const HOOK_OK: HookHealth = HookHealth {
    watched: true,
    reinstalls: 0,
    last_reinstall: None,
    last_attempt_failed: false,
    timer_failing: false,
};

impl TestControl {
    fn new(hook: HookHealth) -> Self {
        Self {
            enabled: AtomicBool::new(true),
            bindings: 2,
            hook,
            toggles: Mutex::new(Vec::new()),
        }
    }
}

impl DaemonControl for TestControl {
    fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        self.enabled.store(enabled, Ordering::Release);
        self.toggles
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(enabled);
        Ok(())
    }

    fn status(&self) -> DaemonStatus {
        DaemonStatus {
            enabled: self.enabled.load(Ordering::Acquire),
            bindings: self.bindings,
            hook: self.hook,
        }
    }

    fn request_exit(&self) -> Result<(), String> {
        Err("the test daemon does not exit".to_owned())
    }
}

#[test]
fn status_off_on_status_round_trip_without_a_hook() {
    let name = unique_pipe_name("server");
    let control = Arc::new(TestControl::new(HOOK_OK));
    ipc_server::bind(&name)
        .expect("the pipe can be created under its own name")
        .serve(control.clone())
        .expect("the server thread starts");
    let send = |command: Command| {
        request(&name, &command, REPLY_TIMEOUT)
            .unwrap_or_else(|error| panic!("{command:?}: no reply: {error}"))
    };

    assert_eq!(
        send(Command::Status),
        (
            true,
            "running (2 bindings live; hook ok, reinstalls: 0)".to_owned()
        )
    );
    assert_eq!(send(Command::Off), (true, "disabled".to_owned()));
    assert!(!control.enabled.load(Ordering::Acquire));
    assert_eq!(
        send(Command::Status),
        (
            true,
            "disabled (2 bindings live; hook ok, reinstalls: 0)".to_owned()
        )
    );
    assert_eq!(send(Command::On), (true, "enabled".to_owned()));
    assert_eq!(
        send(Command::Status),
        (
            true,
            "running (2 bindings live; hook ok, reinstalls: 0)".to_owned()
        )
    );
    assert_eq!(
        *control
            .toggles
            .lock()
            .unwrap_or_else(PoisonError::into_inner),
        [false, true]
    );
}

#[test]
fn status_reports_the_hook_health_over_the_pipe() {
    // Roadmap item 2.4: one reinstall with its time. The reply is still one
    // line that starts as swch and the probe of `swch open` expect.
    let name = unique_pipe_name("health");
    let at = UNIX_EPOCH + Duration::from_millis(1_760_866_867_123);
    let control = Arc::new(TestControl::new(HookHealth {
        reinstalls: 1,
        last_reinstall: Some(at),
        ..HOOK_OK
    }));
    ipc_server::bind(&name)
        .expect("the pipe can be created under its own name")
        .serve(control)
        .expect("the server thread starts");

    let (ok, message) =
        request(&name, &Command::Status, REPLY_TIMEOUT).expect("the server answers");
    assert!(ok);
    assert!(message.starts_with("running (2 bindings live"), "{message}");
    assert_eq!(
        message,
        "running (2 bindings live; hook ok, reinstalls: 1, last 2025-10-19T09:41:07.123Z)"
    );
}

#[test]
fn request_times_out_against_a_server_that_never_replies() {
    assert_eq!(REPLY_TIMEOUT, Duration::from_secs(5));
    let name = unique_pipe_name("silent");
    let listener = listener(&name);
    thread::spawn(move || {
        // Accepts one connection and holds it without reading or writing.
        let _connection = listener.accept();
        thread::sleep(Duration::from_secs(30));
    });

    let start = Instant::now();
    let result = request(&name, &Command::Status, REPLY_TIMEOUT);
    let elapsed = start.elapsed();

    assert!(
        matches!(result, Err(RequestError::TimedOut(_))),
        "a server that never replies times out, got {result:?}"
    );
    assert!(
        elapsed >= REPLY_TIMEOUT - Duration::from_millis(50),
        "gave up early, after {elapsed:?}"
    );
    assert!(
        elapsed < REPLY_TIMEOUT + Duration::from_secs(2),
        "gave up late, after {elapsed:?}"
    );
}

#[test]
fn request_reports_not_running_without_a_pipe() {
    let name = unique_pipe_name("none");
    let start = Instant::now();
    let error = request(&name, &Command::Status, REPLY_TIMEOUT).expect_err("no server listens");
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "a missing pipe is reported at once, not after {:?}",
        start.elapsed()
    );
    assert!(matches!(error, RequestError::NotRunning), "{error:?}");
    assert_eq!(error.daemon_running(), Some(false));
}

#[test]
fn request_reports_access_denied_for_a_pipe_this_process_may_not_open() {
    let name = unique_pipe_name("denied");
    let sddl = "D:P(A;;GA;;;SY)"
        .to_wtf_16()
        .expect("the SDDL has no interior nul");
    let only_system = SecurityDescriptor::deserialize(&sddl).expect("the SDDL parses");
    let _listener = listener_options(&name)
        .security_descriptor(only_system)
        .create_sync()
        .expect("the test pipe can be created");

    let error = request(&name, &Command::Status, REPLY_TIMEOUT)
        .expect_err("only LocalSystem may open the pipe");
    assert!(matches!(error, RequestError::AccessDenied), "{error:?}");
    assert_eq!(error.daemon_running(), Some(true));
}

#[test]
fn request_against_a_pipe_with_no_free_instance_reports_the_daemon_running() {
    let name = unique_pipe_name("busy");
    let path = pipe_path(&name);
    let _listener = PipeListenerOptions::new()
        .path(path.as_str())
        .instance_limit(NonZeroU8::new(1))
        .create_duplex::<pipe_mode::Bytes>()
        .expect("the test pipe can be created");
    let _first = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(path.as_str())
        .expect("the first client takes the only instance");

    let start = Instant::now();
    let error = request(&name, &Command::Status, Duration::from_millis(500))
        .expect_err("no instance is free");
    assert!(
        start.elapsed() < Duration::from_secs(3),
        "the wait for a free instance is bounded, it took {:?}",
        start.elapsed()
    );
    assert!(
        matches!(error, RequestError::Busy | RequestError::TimedOut(_)),
        "{error:?}"
    );
    assert_eq!(error.daemon_running(), Some(true));
}

#[test]
fn request_reads_the_reply_of_a_server_that_answers() {
    let name = unique_pipe_name("answers");
    let listener = listener(&name);
    thread::spawn(move || {
        let connection = listener.accept().expect("a client connects");
        let mut reader = BufReader::new(connection);
        let mut line = String::new();
        reader.read_line(&mut line).expect("the request arrives");
        let mut connection = reader.into_inner();
        connection
            .write_all(format!("OK: got {}\n", line.trim()).as_bytes())
            .expect("the reply is written");
        let _ = connection.flush();
    });

    assert_eq!(
        request(&name, &Command::Status, REPLY_TIMEOUT).expect("the server answers"),
        (true, "got status".to_owned())
    );
}
