//! Explicit, bounded, read-only evdev capture. Never grabs or injects events.
use serde::Serialize;
use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    os::{
        fd::AsRawFd,
        unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt},
    },
    path::Path,
    time::{Duration, Instant},
};

const MAX_EVENTS: usize = 20_000;
#[derive(Debug, Serialize)]
pub struct CapturedEvent {
    pub elapsed_us: u64,
    pub interface: usize,
    pub event_type: u16,
    pub code: u16,
    pub value: i32,
}
#[derive(Debug, Serialize)]
pub struct Capture {
    pub schema_version: u32,
    pub read_only: bool,
    pub duration_ms: u64,
    pub events: Vec<CapturedEvent>,
    pub truncated: bool,
}
#[repr(C)]
#[derive(Default)]
struct InputId {
    bus: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

pub fn capture(paths: &[String], product: u16, seconds: u64) -> io::Result<Capture> {
    if paths.is_empty() || paths.len() > 32 || !(1..=30).contains(&seconds) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "capture needs 1–32 interfaces and 1–30 seconds",
        ));
    }
    let mut files = vec![];
    for path in paths {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(path)?;
        let metadata = file.metadata()?;
        if !metadata.file_type().is_char_device() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "not a character device",
            ));
        }
        let mut id = InputId::default();
        // SAFETY: EVIOCGID writes exactly one repr(C) input_id to a live pointer.
        let result =
            unsafe { libc::ioctl(file.as_raw_fd(), 0x80084502_u64 as libc::c_ulong, &mut id) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        if id.bus != 3 || id.vendor != 0x046d || id.product != product {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "opened interface identity does not match selected wired Logitech model",
            ));
        }
        let major = libc::major(metadata.rdev());
        let minor = libc::minor(metadata.rdev());
        let device = std::fs::canonicalize(format!("/sys/dev/char/{major}:{minor}/device"))?;
        let usb = device
            .ancestors()
            .find(|p| p.join("idVendor").is_file() && p.join("idProduct").is_file())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "opened device has no USB parent",
                )
            })?;
        let identity = |name| std::fs::read_to_string(usb.join(name)).map(|s| s.trim().to_string());
        if identity("idVendor")? != "046d" || identity("idProduct")? != format!("{product:04x}") {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "USB parent identity mismatch",
            ));
        }
        files.push(file);
    }
    capture_open(&mut files, seconds)
}
fn capture_open(files: &mut [File], seconds: u64) -> io::Result<Capture> {
    let start = Instant::now();
    let duration = Duration::from_secs(seconds);
    let size = std::mem::size_of::<libc::timeval>() + 8;
    let mut pending = vec![Vec::<u8>::new(); files.len()];
    let mut events = vec![];
    while start.elapsed() < duration && events.len() < MAX_EVENTS {
        let mut polls: Vec<_> = files
            .iter()
            .map(|f| libc::pollfd {
                fd: f.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            })
            .collect();
        let timeout = duration
            .saturating_sub(start.elapsed())
            .as_millis()
            .min(100) as i32;
        // SAFETY: polls is a live contiguous array with the supplied element count.
        let result =
            unsafe { libc::poll(polls.as_mut_ptr(), polls.len() as libc::nfds_t, timeout) };
        if result < 0 {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(e);
        }
        for (index, poll) in polls.iter().enumerate() {
            if poll.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "capture interface disconnected",
                ));
            }
            if poll.revents & libc::POLLIN == 0 {
                continue;
            }
            let mut buffer = [0; 4096];
            match files[index].read(&mut buffer) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "capture interface closed",
                    ))
                }
                Ok(n) => pending[index].extend_from_slice(&buffer[..n]),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(e),
            }
            while pending[index].len() >= size && events.len() < MAX_EVENTS {
                let bytes = &pending[index][size - 8..size];
                events.push(CapturedEvent {
                    elapsed_us: start.elapsed().as_micros() as u64,
                    interface: index,
                    event_type: u16::from_ne_bytes([bytes[0], bytes[1]]),
                    code: u16::from_ne_bytes([bytes[2], bytes[3]]),
                    value: i32::from_ne_bytes(bytes[4..8].try_into().unwrap()),
                });
                pending[index].drain(..size);
            }
        }
    }
    Ok(Capture {
        schema_version: 1,
        read_only: true,
        duration_ms: start.elapsed().as_millis() as u64,
        truncated: events.len() >= MAX_EVENTS,
        events,
    })
}

pub fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
