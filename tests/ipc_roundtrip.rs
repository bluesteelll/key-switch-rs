//! The `swch` client (`ipc::request`) against test pipes with unique names.
//!
//! No test here installs a keyboard hook or talks to a daemon: every pipe
//! is created by the test itself, under a name no daemon uses.

use std::io::{BufRead, BufReader, Write};
use std::num::NonZeroU8;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use interprocess::local_socket::{GenericNamespaced, Listener, ListenerOptions, prelude::*};
use interprocess::os::windows::ToWtf16;
use interprocess::os::windows::local_socket::ListenerOptionsExt;
use interprocess::os::windows::named_pipe::{DuplexPipeStream, PipeListenerOptions, pipe_mode};
use interprocess::os::windows::security_descriptor::SecurityDescriptor;

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
