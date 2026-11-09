//! Single instance (WR-09 AC1). The first instance listens on a local socket. A second
//! instance connects, sends `open\n` and its `version <x.y.z>\n`, and exits 0 once the first
//! answers `stay\n`. When the second copy is a newer version, the first answers `quit\n` and ends,
//! and the second takes the socket over and starts: starting a newer version never leaves an
//! older one running. A copy from before the version line sends only `open\n` and closes; the
//! first then just opens. The listener runs on a background thread and wakes the UI thread with
//! `slint::invoke_from_event_loop`, because a Slint component is not `Send`. The listener
//! replaced the 250 ms polling timer, so an idle app has no four-times-a-second UI wake.
//!
//! The name is `mitibar-<username-hash>.sock` in the OS namespace when available (Windows,
//! Linux). macOS has no namespace, so the socket is a file in the data-local directory. A stale
//! file is removed and the listener retried.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::time::Duration;

use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, Listener, ListenerOptions, Stream,
};

use crate::app::version::{self, Version};

/// What this process is.
pub enum Claim {
    /// The first instance, or a newer copy that took over from an older one; keep the listener.
    Primary(Listener),
    /// A copy of the same or a newer version is running; it was told to open.
    Secondary,
}

/// What the running instance answers a second one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Answer {
    /// The running copy opens its popup and keeps running.
    Stay,
    /// The running copy is older and ends; the caller takes over.
    Quit,
}

/// What one connection asked of the running instance.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Request {
    Open,
    /// A newer copy started; this one should end.
    Replace(Version),
}

/// The socket address, kept as data so the `Name` can borrow it locally.
enum Address {
    Namespaced(String),
    File(PathBuf),
}

/// Claims the app's socket, or signals the running instance.
///
/// # Errors
///
/// Returns an error when the socket cannot be created or an existing one cannot be checked.
pub fn claim() -> anyhow::Result<Claim> {
    let address = address()?;
    claim_at(&address, &version::current())
}

/// Starts the listener thread. Each `open\n` from a copy of the same or an older version calls
/// `wake`, which the composition root turns into `slint::invoke_from_event_loop(run_open_request)`.
/// A newer copy gets `quit\n`, the listener closes and `newer` runs, so the app can end. Both
/// callbacks must be `Send`; they capture no Slint type.
pub fn start(
    listener: Listener,
    wake: impl Fn() + Send + 'static,
    newer: impl FnOnce(&Version) + Send + 'static,
) {
    let mine = version::current();
    std::thread::spawn(move || {
        for connection in listener.incoming() {
            let Ok(stream) = connection else {
                break;
            };
            match serve(stream, &mine) {
                Some(Request::Open) => wake(),
                Some(Request::Replace(theirs)) => {
                    newer(&theirs);
                    // Ending the loop drops the listener, which frees the name for the newer copy.
                    break;
                }
                None => {}
            }
        }
    });
}

/// The most bytes read for one line; `version 65535.65535.65535-beta.99` fits.
const MESSAGE_LIMIT: usize = 48;

/// How long a newer copy waits for the older one to let go of the socket, and how often it looks.
const TAKE_OVER_WAIT: Duration = Duration::from_secs(5);
const TAKE_OVER_STEP: Duration = Duration::from_millis(100);

/// Reads one connection, `open` and then the optional version line, and answers a caller that
/// sent its version. Anything else is not ours and gets no answer.
fn serve(stream: Stream, mine: &Version) -> Option<Request> {
    let mut reader = BufReader::with_capacity(MESSAGE_LIMIT, stream);
    if read_line(&mut reader)? != "open" {
        return None;
    }
    // A copy from before the version line sends `open` alone and closes, so this reads nothing.
    let theirs = read_line(&mut reader)
        .and_then(|line| line.strip_prefix("version ").and_then(Version::parse));
    let request = match &theirs {
        Some(theirs) if theirs > mine => Request::Replace(theirs.clone()),
        _ => Request::Open,
    };
    if theirs.is_some() {
        let answer: &[u8] = if request == Request::Open {
            b"stay\n"
        } else {
            b"quit\n"
        };
        let stream = reader.get_mut();
        // The caller may already be gone; it then simply exits, as a second copy always did.
        let _ = stream.write_all(answer).and_then(|()| stream.flush());
    }
    Some(request)
}

/// One trimmed line of at most `MESSAGE_LIMIT` bytes; `None` at the end of the stream.
fn read_line(reader: &mut BufReader<Stream>) -> Option<String> {
    let mut line = String::new();
    let limit = u64::try_from(MESSAGE_LIMIT).unwrap_or(u64::MAX);
    match reader.by_ref().take(limit).read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line.trim().to_owned()),
    }
}

fn claim_at(address: &Address, mine: &Version) -> anyhow::Result<Claim> {
    if let Ok(listener) = create(address) {
        return Ok(Claim::Primary(listener));
    }
    match signal_at(address, mine) {
        Ok(Answer::Stay) => return Ok(Claim::Secondary),
        Ok(Answer::Quit) => return Ok(take_over(address)),
        Err(_) => {}
    }
    remove_stale(address);
    let listener = create(address)?;
    Ok(Claim::Primary(listener))
}

/// Waits for the older copy to end and claims the socket. If it does not let go in time, this copy
/// gives way, as a second copy always did, and the older one keeps running.
fn take_over(address: &Address) -> Claim {
    let mut waited = Duration::ZERO;
    while waited < TAKE_OVER_WAIT {
        std::thread::sleep(TAKE_OVER_STEP);
        waited += TAKE_OVER_STEP;
        if let Ok(listener) = create(address) {
            log::info!("single instance: took over from an older copy");
            return Claim::Primary(listener);
        }
    }
    log::warn!("single instance: the older copy did not end; leaving it running");
    Claim::Secondary
}

fn address() -> anyhow::Result<Address> {
    let name = format!("mitibar-{:016x}.sock", user_hash());
    if GenericNamespaced::is_supported() {
        return Ok(Address::Namespaced(name));
    }
    let dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("no data-local directory for the socket"))?
        .join("mitibar");
    std::fs::create_dir_all(&dir)?;
    Ok(Address::File(dir.join(name)))
}

fn user_hash() -> u64 {
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default();
    fnv1a(user.as_bytes())
}

/// 64-bit FNV-1a. The socket name must be the same in every build of the app, so an upgraded
/// binary still finds the running one; `DefaultHasher` makes no such promise across Rust
/// releases.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn create(address: &Address) -> anyhow::Result<Listener> {
    let options = match address {
        Address::Namespaced(name) => {
            let name = name.as_str().to_ns_name::<GenericNamespaced>()?;
            ListenerOptions::new().name(name)
        }
        Address::File(path) => {
            let name = path.as_path().to_fs_name::<GenericFilePath>()?;
            ListenerOptions::new().name(name)
        }
    };
    Ok(options.create_sync()?)
}

fn connect(address: &Address) -> anyhow::Result<Stream> {
    Ok(match address {
        Address::Namespaced(name) => {
            let name = name.as_str().to_ns_name::<GenericNamespaced>()?;
            Stream::connect(name)?
        }
        Address::File(path) => {
            let name = path.as_path().to_fs_name::<GenericFilePath>()?;
            Stream::connect(name)?
        }
    })
}

fn signal_at(address: &Address, mine: &Version) -> anyhow::Result<Answer> {
    let mut stream = connect(address)?;
    stream.write_all(format!("open\nversion {mine}\n").as_bytes())?;
    stream.flush()?;
    // A copy from before the version line closes without an answer: it opened and stays.
    let mut reader = BufReader::with_capacity(MESSAGE_LIMIT, stream);
    Ok(match read_line(&mut reader).as_deref() {
        Some("quit") => Answer::Quit,
        _ => Answer::Stay,
    })
}

fn remove_stale(address: &Address) {
    if let Address::File(path) = address {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique test socket. Windows uses a named pipe; Unix uses a file in a tempdir, which the
    /// caller removes.
    #[cfg(windows)]
    fn test_address(tag: &str) -> (Address, Option<tempfile::TempDir>) {
        (
            Address::Namespaced(format!("mitibar-test-{tag}-{}", std::process::id())),
            None,
        )
    }

    #[cfg(not(windows))]
    fn test_address(tag: &str) -> (Address, Option<tempfile::TempDir>) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(format!("mitibar-test-{tag}.sock"));
        (Address::File(path), Some(dir))
    }

    fn v(text: &str) -> Version {
        Version::parse(text).expect(text)
    }

    /// A running instance of version `mine` that reports what each connection asked for, and ends
    /// after a `Replace` the way the app does.
    fn running(address: &Address, mine: &str) -> std::sync::mpsc::Receiver<Option<Request>> {
        let listener = create(address).expect("listener");
        let mine = v(mine);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for connection in listener.incoming() {
                let Ok(stream) = connection else { break };
                let request = serve(stream, &mine);
                let replace = matches!(request, Some(Request::Replace(_)));
                let _ = tx.send(request);
                if replace {
                    break;
                }
            }
        });
        rx
    }

    fn next(served: &std::sync::mpsc::Receiver<Option<Request>>) -> Option<Request> {
        served
            .recv_timeout(Duration::from_secs(2))
            .expect("the running instance served the connection")
    }

    #[test]
    fn socket_name_hash_is_fixed_across_builds() {
        // Published FNV-1a 64 test vectors.
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn wr09_ac1_second_instance_signals_the_first() {
        let (address, _dir) = test_address("signal");
        let mine = v("1.0.0");
        let Claim::Primary(listener) = claim_at(&address, &mine).expect("primary") else {
            panic!("first claim must be primary");
        };
        let (wake, woken) = std::sync::mpsc::channel();
        start(
            listener,
            move || {
                let _ = wake.send(());
            },
            |_| panic!("the same version never replaces the running copy"),
        );
        assert_eq!(signal_at(&address, &mine).expect("signal"), Answer::Stay);
        woken
            .recv_timeout(Duration::from_secs(2))
            .expect("the listener woke the UI callback");
        // The thread owns the listener for the rest of this process; the test process ends here.
    }

    #[test]
    fn an_older_copy_started_later_only_opens_the_running_newer_one() {
        let (address, _dir) = test_address("older");
        let served = running(&address, "1.1.0");
        assert!(matches!(
            claim_at(&address, &v("1.0.0")).expect("claim"),
            Claim::Secondary
        ));
        assert_eq!(next(&served), Some(Request::Open));
    }

    #[test]
    fn a_newer_copy_takes_over_from_a_running_older_one() {
        let (address, _dir) = test_address("newer");
        let served = running(&address, "0.2.0-beta.2");
        let Claim::Primary(_listener) = claim_at(&address, &v("1.0.0")).expect("claim") else {
            panic!("the newer copy must take over");
        };
        assert_eq!(next(&served), Some(Request::Replace(v("1.0.0"))));
    }

    #[test]
    fn a_copy_from_before_the_version_line_still_opens_the_running_one() {
        let (address, _dir) = test_address("legacy");
        let served = running(&address, "1.1.0");
        let mut stream = connect(&address).expect("connect");
        stream.write_all(b"open\n").expect("write");
        drop(stream);
        assert_eq!(next(&served), Some(Request::Open));
    }

    #[test]
    fn a_stranger_on_the_socket_is_ignored() {
        let (address, _dir) = test_address("stranger");
        let served = running(&address, "1.1.0");
        let mut stream = connect(&address).expect("connect");
        stream.write_all(b"GET / HTTP/1.1\n").expect("write");
        drop(stream);
        assert_eq!(next(&served), None);
    }

    #[cfg(not(windows))]
    #[test]
    fn wr09_ac1_stale_socket_is_taken_over() {
        let (address, _dir) = test_address("stale");
        if let Address::File(path) = &address {
            std::fs::write(path, b"stale").expect("stale file");
        }
        let Claim::Primary(listener) = claim_at(&address, &v("1.0.0")).expect("take over") else {
            panic!("a stale file must not count as a running instance");
        };
        drop(listener);
        if let Address::File(path) = &address {
            let _ = std::fs::remove_file(path);
            assert!(!path.exists(), "test socket cleaned up");
        }
    }
}
