use std::fs::{self, Permissions};
use std::io::{self, Read};
use std::mem;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::sync::Mutex;
use std::thread;

pub const SOCKET: &str = "/tmp/sunlight.sock";

static QUEUE: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

/// Queues each connection's contents as one script. Nothing is sent back,
/// since output and errors already reach the Roblox log; write a status to
/// the stream if a client ever needs an exit code.
pub fn listen() -> io::Result<()> {
    let _ = fs::remove_file(SOCKET);
    let listener = UnixListener::bind(SOCKET)?;
    fs::set_permissions(SOCKET, Permissions::from_mode(0o600))?;
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut source = Vec::new();
            if stream.and_then(|mut s| s.read_to_end(&mut source)).is_ok() {
                QUEUE.lock().unwrap().push(source);
            }
        }
    });
    Ok(())
}

pub fn take() -> Vec<Vec<u8>> {
    mem::take(&mut QUEUE.lock().unwrap())
}
