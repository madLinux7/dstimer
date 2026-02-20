use crate::render::is_quit_event;
use crossterm::event::{self, Event};
use rodio::{Decoder, OutputStream, Sink};
use std::{
    io::BufReader,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread,
    time::Duration,
};

/// Temporarily redirect stderr to /dev/null for the duration of `f`.
/// Needed on Linux to suppress noisy JACK/ALSA fallback messages from
/// cpal when it probes unavailable backends before settling on a working one.
#[cfg(target_os = "linux")]
fn with_stderr_suppressed<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    use std::os::unix::io::IntoRawFd;
    unsafe {
        let saved = libc::dup(2);
        let devnull = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/null")
            .map(|f| f.into_raw_fd())
            .unwrap_or(-1);
        if devnull >= 0 {
            libc::dup2(devnull, 2);
            libc::close(devnull);
        }
        let result = f();
        if saved >= 0 {
            libc::dup2(saved, 2);
            libc::close(saved);
        }
        result
    }
}

#[cfg(not(target_os = "linux"))]
fn with_stderr_suppressed<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    f()
}

pub fn play_audio(path: &Path, running: &Arc<AtomicBool>) {
    let Ok((_stream, stream_handle)) = with_stderr_suppressed(OutputStream::try_default) else {
        eprintln!("Error: Could not open audio output device.");
        return;
    };

    let Ok(file) = std::fs::File::open(path) else {
        eprintln!("Error: Could not open audio file: {}", path.display());
        return;
    };

    let Ok(source) = Decoder::new(BufReader::new(file)) else {
        eprintln!("Error: Could not decode audio file: {}", path.display());
        return;
    };

    let Ok(sink) = Sink::try_new(&stream_handle) else {
        eprintln!("Error: Could not create audio sink.");
        return;
    };

    sink.append(source);

    // Poll crossterm key events because raw mode suppresses SIGINT.
    while !sink.empty() && running.load(Ordering::Relaxed) {
        if event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = event::read() {
                if is_quit_event(&key_event) {
                    running.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    sink.stop();
}
