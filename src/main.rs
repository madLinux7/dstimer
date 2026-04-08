mod audio;
mod config;
mod render;

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event},
    style::{Color, Print},
    terminal::{self, size, Clear, ClearType},
    ExecutableCommand,
};
#[cfg(not(target_os = "macos"))]
use notify_rust::Notification;

use std::{
    io::{self, stdout},
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

pub const ERR_ZERO_DURATION: &str = "Duration must be greater than 0 seconds";

#[derive(Parser)]
#[command(name = "dstimer")]
#[command(
    about = "A centered CLI timer with color-changing progress bar and option to play audio on finish."
)]
struct Args {
    /// Duration in HH:MM:SS format (e.g. 1:30:00, 5:00, 90)
    #[arg(short, long, value_parser = parse_time)]
    time: Option<u64>,

    /// Preset name or duration (e.g. "pomodoro" or "5:00")
    #[arg()]
    positional: Option<String>,

    /// Optional path to audio file to play when timer completes
    #[arg(short, long)]
    audio: Option<PathBuf>,

    /// Optional URL to call (HTTP GET) when timer completes
    #[arg(short, long)]
    url: Option<String>,

    /// Disable notifications when timer finishes
    #[arg(long)]
    silent: bool,

    /// Inline mode: show timer on current line without clearing the screen
    #[arg(short, long)]
    inline: bool,
}

/// Parse a time string in HH:MM:SS, MM:SS, or SS format into total seconds.
fn parse_time(s: &str) -> Result<u64, String> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        1 => parts[0].parse::<u64>().map_err(|e| e.to_string()),
        2 => {
            let mins = parts[0].parse::<u64>().map_err(|e| e.to_string())?;
            let secs = parts[1].parse::<u64>().map_err(|e| e.to_string())?;
            if secs >= 60 {
                return Err("seconds must be 0-59".to_string());
            }
            Ok(mins * 60 + secs)
        }
        3 => {
            let hrs = parts[0].parse::<u64>().map_err(|e| e.to_string())?;
            let mins = parts[1].parse::<u64>().map_err(|e| e.to_string())?;
            let secs = parts[2].parse::<u64>().map_err(|e| e.to_string())?;
            if mins >= 60 {
                return Err("minutes must be 0-59".to_string());
            }
            if secs >= 60 {
                return Err("seconds must be 0-59".to_string());
            }
            Ok(hrs * 3600 + mins * 60 + secs)
        }
        _ => Err("expected format: HH:MM:SS, MM:SS, or SS".to_string()),
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let defaults = config::load_defaults();
    let presets = config::load_presets();

    // Resolve positional: preset name or time value
    let (preset_name, time_from_positional) = match &args.positional {
        Some(val) => {
            if presets.contains_key(val) {
                (Some(val.as_str()), None)
            } else {
                match parse_time(val) {
                    Ok(t) => (None, Some(t)),
                    Err(e) => {
                        eprintln!("Error: '{val}' is not a known preset and not a valid time: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
        None => (None, None),
    };

    let preset_entry = preset_name.and_then(|name| presets.get(name));
    let eff = config::merge(
        &defaults,
        preset_entry,
        args.inline,
        args.silent,
        args.audio,
        args.url,
    );

    // Validate audio path if resolved from preset/defaults
    if let Some(ref path) = eff.audio {
        if !path.exists() {
            eprintln!("Error: audio file not found: {}", path.display());
            std::process::exit(1);
        }
    }

    // CLI --time > positional time > preset time > TUI prompt
    let preset_time = eff.time.as_deref().and_then(|s| parse_time(s).ok());
    let explicit_duration = args.time.or(time_from_positional).or(preset_time);
    let has_explicit_duration = explicit_duration.is_some();

    let (duration_secs, audio_path, url) = if let Some(secs) = explicit_duration {
        (secs, eff.audio, eff.url)
    } else {
        // TUI mode — skip prompts for audio/url if already resolved
        if eff.inline {
            render::inline_interactive_prompt(eff.audio, eff.url)?
        } else {
            render::interactive_prompt(eff.audio, eff.url)?
        }
    };

    if duration_secs == 0 {
        println!("{ERR_ZERO_DURATION}");
        return Ok(());
    }

    let duration = Duration::from_secs(duration_secs);
    let start = Instant::now();
    let end = start + duration;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    let mut stdout = stdout();

    if eff.inline {
        if has_explicit_duration {
            stdout.execute(Print("\r\n"))?;
        }

        run_inline_timer(
            &mut stdout,
            duration,
            start,
            end,
            &running,
            &audio_path,
            &url,
            eff.silent,
        )?;
    } else {
        run_fullscreen_timer(
            &mut stdout,
            duration,
            start,
            end,
            &running,
            &audio_path,
            &url,
            eff.silent,
        )?;
    }

    Ok(())
}

fn get_color(progress: f64) -> Color {
    if progress < 0.5 {
        Color::Green
    } else if progress < 0.8 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn run_fullscreen_timer(
    stdout: &mut io::Stdout,
    duration: Duration,
    start: Instant,
    end: Instant,
    running: &Arc<AtomicBool>,
    audio_path: &Option<PathBuf>,
    url: &Option<String>,
    silent: bool,
) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    stdout.execute(cursor::Hide)?;
    stdout.execute(Clear(ClearType::All))?;

    while running.load(Ordering::Relaxed) && Instant::now() < end {
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                if render::is_quit_event(&key_event) {
                    running.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }

        let elapsed = start.elapsed();
        let remaining = duration.saturating_sub(elapsed);
        let progress = elapsed.as_secs_f64() / duration.as_secs_f64();

        render::draw_timer(stdout, remaining, progress, get_color(progress))?;
        thread::sleep(Duration::from_millis(50));
    }

    let (_, rows) = size()?;
    let center_row = rows / 2;

    handle_finish(
        stdout,
        running,
        end,
        audio_path,
        url,
        silent,
        |stdout, msg| render::print_centered(stdout, center_row + 2, msg),
        |stdout, msg| render::print_centered(stdout, center_row + 4, msg),
    )?;

    stdout.execute(cursor::Show)?;
    let (_, rows) = size()?;
    stdout.execute(cursor::MoveTo(0, rows - 1))?;
    stdout.execute(Print("\n"))?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn run_inline_timer(
    stdout: &mut io::Stdout,
    duration: Duration,
    start: Instant,
    end: Instant,
    running: &Arc<AtomicBool>,
    audio_path: &Option<PathBuf>,
    url: &Option<String>,
    silent: bool,
) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    stdout.execute(cursor::Hide)?;

    // Render on the current line (same line the interactive prompt left off on)
    let (_, timer_row) = cursor::position()?;

    while running.load(Ordering::Relaxed) && Instant::now() < end {
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                if render::is_quit_event(&key_event) {
                    running.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }

        let elapsed = start.elapsed();
        let remaining = duration.saturating_sub(elapsed);
        let progress = elapsed.as_secs_f64() / duration.as_secs_f64();

        render::draw_inline_timer(stdout, timer_row, remaining, progress, get_color(progress))?;
        thread::sleep(Duration::from_millis(50));
    }

    handle_finish(
        stdout,
        running,
        end,
        audio_path,
        url,
        silent,
        |stdout, msg| render::print_inline_finish(stdout, timer_row, msg),
        |stdout, msg| render::print_inline_finish(stdout, timer_row + 1, msg),
    )?;

    // Blank line after finish message
    stdout.execute(Print("\r\n\r\n"))?;
    stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn handle_finish<F, G>(
    stdout: &mut io::Stdout,
    running: &Arc<AtomicBool>,
    end: Instant,
    audio_path: &Option<PathBuf>,
    url: &Option<String>,
    silent: bool,
    print_msg: F,
    print_url_msg: G,
) -> io::Result<()>
where
    F: Fn(&mut io::Stdout, &str) -> io::Result<()>,
    G: Fn(&mut io::Stdout, &str) -> io::Result<()>,
{
    if running.load(Ordering::Relaxed) && Instant::now() >= end {
        const FINISHED_MSG: &str = "Timer finished!";
        const FINISHED_MSG_AUDIO: &str = "Timer finished \u{266a} Playing audio";

        let has_audio = audio_path.is_some();
        let body = if has_audio {
            FINISHED_MSG_AUDIO
        } else {
            FINISHED_MSG
        };
        if !silent {
            send_notification("Dead Simple CLI Timer", body);
        }

        if has_audio {
            print_msg(stdout, FINISHED_MSG_AUDIO)?;
            audio::play_audio(audio_path.as_ref().unwrap(), running);
        } else {
            print_msg(stdout, FINISHED_MSG)?;
        }

        if let Some(ref url) = url {
            fire_url(url, &print_url_msg, stdout)?;
        }
    } else {
        print_msg(stdout, "Timer cancelled")?;
    }
    Ok(())
}

fn fire_url<F>(url: &str, print_msg: &F, stdout: &mut io::Stdout) -> io::Result<()>
where
    F: Fn(&mut io::Stdout, &str) -> io::Result<()>,
{
    print_msg(stdout, &format!("Calling URL..."))?;
    match ureq::get(url).call() {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.into_string().unwrap_or_default();
            if body.is_empty() {
                print_msg(stdout, &format!("HTTP {status} (no body)"))?;
            } else {
                print_msg(stdout, &format!("HTTP {status}: {body}"))?;
            }
        }
        Err(e) => {
            print_msg(stdout, &format!("HTTP error: {e}"))?;
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn send_notification(title: &str, body: &str) {
    print!("\x07");
    let script = format!(
        "display notification \"{}\" with title \"{}\" sound name \"default\"",
        body, title
    );
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output();
}

#[cfg(not(target_os = "macos"))]
fn send_notification(title: &str, body: &str) {
    print!("\x07");
    let _ = Notification::new()
        .summary(title)
        .body(body)
        .appname("dstimer")
        .show();
}
