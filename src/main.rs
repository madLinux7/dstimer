mod audio;
mod render;

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event},
    style::{Color, Print},
    terminal::{self, size, Clear, ClearType},
    ExecutableCommand,
};
use notify_rust::Notification;
use std::{
    io::{self, stdout},
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(name = "dstimer")]
#[command(
    about = "A centered CLI timer with color-changing progress bar and option to play audio on finish."
)]
struct Args {
    /// Duration in seconds
    #[arg(short, long)]
    seconds: Option<u64>,

    /// Optional path to audio file to play when timer completes
    #[arg(short, long)]
    audio: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let (duration_secs, audio_path) = if args.seconds.is_none() {
        render::interactive_prompt()?
    } else {
        if let Some(ref path) = args.audio {
            if !path.exists() {
                eprintln!("Error: audio file not found: {}", path.display());
                std::process::exit(1);
            }
        }
        (args.seconds.unwrap(), args.audio)
    };

    if duration_secs == 0 {
        println!("Duration must be greater than 0.");
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

        let color = if progress < 0.5 {
            Color::Green
        } else if progress < 0.8 {
            Color::Yellow
        } else {
            Color::Red
        };

        render::draw_timer(&mut stdout, remaining, progress, color)?;
        thread::sleep(Duration::from_millis(50));
    }

    // Only play audio if timer completed naturally (not interrupted)
    if running.load(Ordering::Relaxed) && Instant::now() >= end {
        const FINISHED_MSG: &str = "Timer finished!";
        const FINISHED_MSG_AUDIO: &str = "Timer finished ♪ Playing audio";

        let audio_used = audio_path.is_some();
        let _ = Notification::new()
            .summary("Dead Simple CLI Timer")
            .body(if audio_used { FINISHED_MSG_AUDIO } else { FINISHED_MSG })
            .appname("dstimer")
            .show();

        let (_, rows) = size()?;
        let center_row = rows / 2;

        if audio_used {
            render::print_centered(
                &mut stdout,
                center_row + 2,
                FINISHED_MSG_AUDIO,
            )?;
            audio::play_audio(audio_path.as_ref().unwrap(), &running);
        } else {
            render::print_centered(&mut stdout, center_row + 2, FINISHED_MSG)?;
        }
    } else {
        println!("Timer cancelled.");
    }

    // Cleanup: show cursor, move to bottom, disable raw mode
    stdout.execute(cursor::Show)?;
    let (_, rows) = size()?;
    stdout.execute(cursor::MoveTo(0, rows - 1))?;
    stdout.execute(Print("\n"))?;
    terminal::disable_raw_mode()?;

    Ok(())
}
