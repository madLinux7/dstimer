use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyModifiers},
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, size, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::{
    io::{self, stdout, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

/// Returns true if the event is Ctrl+C or Esc.
pub fn is_quit_event(key: &event::KeyEvent) -> bool {
    key.code == KeyCode::Esc
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}

/// Clean up terminal state and exit.
fn cleanup_and_exit(stdout: &mut io::Stdout) -> ! {
    let _ = stdout.execute(Clear(ClearType::All));
    let _ = stdout.execute(cursor::MoveTo(0, 0));
    let _ = stdout.execute(cursor::Show);
    let _ = terminal::disable_raw_mode();
    std::process::exit(0);
}

/// Print a message centered horizontally at the given row.
pub fn print_centered(stdout: &mut io::Stdout, row: u16, msg: &str) -> io::Result<()> {
    let (cols, _) = size()?;
    let col = cols.saturating_sub(msg.len() as u16) / 2;
    stdout.queue(MoveTo(col, row))?.queue(Print(msg))?.flush()
}

pub fn draw_timer(
    stdout: &mut io::Stdout,
    remaining: Duration,
    progress: f64,
    color: Color,
) -> io::Result<()> {
    let (cols, rows) = size()?;

    let total_secs = remaining.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    let time_str = if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    };

    let bar_width = cols.saturating_sub(4);
    let filled = ((1.0 - progress) * bar_width as f64).round().min(bar_width as f64) as u16;
    let empty = bar_width.saturating_sub(filled);

    let bar = format!(
        "{}{}",
        "\u{2588}".repeat(filled as usize),
        "\u{2591}".repeat(empty as usize),
    );

    let center_row = rows / 2;
    let time_col = cols.saturating_sub(time_str.len() as u16) / 2;

    stdout
        .queue(Clear(ClearType::All))?
        .queue(MoveTo(time_col, center_row.saturating_sub(2)))?
        .queue(SetForegroundColor(color))?
        .queue(SetAttribute(Attribute::Bold))?
        .queue(Print(&time_str))?
        .queue(SetAttribute(Attribute::Reset))?
        .queue(MoveTo(2, center_row))?
        .queue(SetForegroundColor(color))?
        .queue(Print(&bar))?
        .queue(ResetColor)?
        .flush()?;

    Ok(())
}

/// Interactive prompt: centered HH:MM:SS digit-by-digit entry.
/// Returns (total_seconds, optional_audio_path).
pub fn interactive_prompt() -> io::Result<(u64, Option<PathBuf>)> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(cursor::Hide)?;
    stdout.execute(Clear(ClearType::All))?;

    let mut digits: [u8; 6] = [0; 6];
    let mut cursor_pos: usize = 0;
    let mut blink_visible = true;
    let mut last_blink = Instant::now();
    let blink_interval = Duration::from_millis(500);

    loop {
        if last_blink.elapsed() >= blink_interval {
            blink_visible = !blink_visible;
            last_blink = Instant::now();
        }

        draw_time_input(&mut stdout, &digits, cursor_pos, blink_visible)?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if is_quit_event(&key_event) {
                    cleanup_and_exit(&mut stdout);
                }
                match key_event.code {
                    KeyCode::Char(c) if c.is_ascii_digit() && cursor_pos < 6 => {
                        let d = c as u8 - b'0';
                        // Only M-tens and S-tens are restricted (0-5)
                        let valid = match cursor_pos {
                            2 | 4 => d <= 5,
                            _ => true,
                        };
                        if valid {
                            digits[cursor_pos] = d;
                            cursor_pos += 1;
                            blink_visible = true;
                            last_blink = Instant::now();
                        }
                    }
                    KeyCode::Backspace if cursor_pos > 0 => {
                        cursor_pos -= 1;
                        digits[cursor_pos] = 0;
                        blink_visible = true;
                        last_blink = Instant::now();
                    }
                    KeyCode::Left if cursor_pos > 0 => {
                        cursor_pos -= 1;
                        blink_visible = true;
                        last_blink = Instant::now();
                    }
                    KeyCode::Right if cursor_pos < 5 => {
                        cursor_pos += 1;
                        blink_visible = true;
                        last_blink = Instant::now();
                    }
                    KeyCode::Enter => break,
                    _ => {}
                }
            }
        }
    }

    let hours = (digits[0] as u64) * 10 + (digits[1] as u64);
    let minutes = (digits[2] as u64) * 10 + (digits[3] as u64);
    let seconds = (digits[4] as u64) * 10 + (digits[5] as u64);
    let total_secs = hours * 3600 + minutes * 60 + seconds;

    let audio_path = prompt_audio_path(&mut stdout)?;

    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok((total_secs, audio_path))
}

/// Draw the HH:MM:SS input mask centered on screen.
fn draw_time_input(
    stdout: &mut io::Stdout,
    digits: &[u8; 6],
    cursor_pos: usize,
    blink_visible: bool,
) -> io::Result<()> {
    let (cols, rows) = size()?;
    let center_row = rows / 2;

    let title = "Enter duration (press Enter to start):";
    let title_col = cols.saturating_sub(title.len() as u16) / 2;

    stdout
        .queue(MoveTo(title_col, center_row.saturating_sub(2)))?
        .queue(Clear(ClearType::CurrentLine))?
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print(title))?
        .queue(ResetColor)?;

    // "HH : MM : SS" = 14 chars
    let start_col = cols.saturating_sub(14) / 2;

    stdout
        .queue(MoveTo(0, center_row))?
        .queue(Clear(ClearType::CurrentLine))?
        .queue(MoveTo(start_col, center_row))?;

    for i in 0..6 {
        if i == 2 || i == 4 {
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(" : "))?;
        }

        let placeholder = match i {
            0 | 1 => "H",
            2 | 3 => "M",
            _ => "S",
        };

        let ch = if digits[i] > 0 || i < cursor_pos {
            format!("{}", digits[i])
        } else if i == cursor_pos {
            if blink_visible {
                placeholder.to_string()
            } else {
                " ".to_string()
            }
        } else {
            placeholder.to_string()
        };

        let color = if i == cursor_pos {
            Color::Cyan
        } else if i < cursor_pos {
            Color::White
        } else {
            Color::DarkGrey
        };

        if i == cursor_pos {
            stdout
                .queue(SetForegroundColor(color))?
                .queue(SetAttribute(Attribute::Bold))?
                .queue(Print(&ch))?
                .queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(SetForegroundColor(color))?.queue(Print(&ch))?;
        }
    }

    let hint = "Esc to quit | \u{2190} \u{2192} to navigate | Backspace to delete";
    let hint_col = cols.saturating_sub(hint.len() as u16) / 2;
    stdout
        .queue(MoveTo(hint_col, center_row + 2))?
        .queue(Clear(ClearType::CurrentLine))?
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print(hint))?
        .queue(ResetColor)?
        .flush()?;

    Ok(())
}

/// Prompt for an optional audio file path.
fn prompt_audio_path(stdout: &mut io::Stdout) -> io::Result<Option<PathBuf>> {
    stdout.execute(Clear(ClearType::All))?;

    let title = "Audio file path (optional, press Enter to skip):";
    let mut input = String::new();
    let mut error_msg: Option<String> = None;

    stdout.execute(cursor::Show)?;

    loop {
        let (cols, rows) = size()?;
        let center_row = rows / 2;
        let title_col = cols.saturating_sub(title.len() as u16) / 2;

        // Error message
        let error_row = center_row.saturating_sub(3);
        stdout
            .queue(MoveTo(0, error_row))?
            .queue(Clear(ClearType::CurrentLine))?;
        if let Some(ref err) = error_msg {
            let err_col = cols.saturating_sub(err.len() as u16) / 2;
            stdout
                .queue(MoveTo(err_col, error_row))?
                .queue(SetForegroundColor(Color::Red))?
                .queue(SetAttribute(Attribute::Bold))?
                .queue(Print(err))?
                .queue(SetAttribute(Attribute::Reset))?
                .queue(ResetColor)?;
        }

        // Title
        stdout
            .queue(MoveTo(title_col, center_row.saturating_sub(1)))?
            .queue(Clear(ClearType::CurrentLine))?
            .queue(SetForegroundColor(Color::DarkGrey))?
            .queue(Print(title))?
            .queue(ResetColor)?;

        // Input
        let display_width = 60.min(cols as usize);
        let input_col = (cols as usize).saturating_sub(display_width) / 2;
        let visible_input = if input.len() > display_width {
            &input[input.len() - display_width..]
        } else {
            &input
        };

        stdout
            .queue(MoveTo(input_col as u16, center_row))?
            .queue(Clear(ClearType::CurrentLine))?
            .queue(SetForegroundColor(Color::Cyan))?
            .queue(SetAttribute(Attribute::Bold))?
            .queue(Print(visible_input))?
            .queue(SetAttribute(Attribute::Reset))?;

        if input.is_empty() {
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(Print("_"))?;
        }

        stdout.queue(ResetColor)?.flush()?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if is_quit_event(&key_event) {
                    cleanup_and_exit(stdout);
                }
                match key_event.code {
                    KeyCode::Enter => {
                        let trimmed = input.trim();
                        if trimmed.is_empty() {
                            stdout.execute(cursor::Hide)?;
                            return Ok(None);
                        }
                        let path = PathBuf::from(trimmed);
                        if path.exists() {
                            stdout.execute(cursor::Hide)?;
                            return Ok(Some(path));
                        }
                        error_msg = Some(format!("\"{}\" not found, check for typos", trimmed));
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        error_msg = None;
                    }
                    KeyCode::Backspace => {
                        input.pop();
                        error_msg = None;
                    }
                    _ => {}
                }
            }
        }
    }
}
