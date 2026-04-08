use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
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
    let time_str = format_time(remaining);

    let bar_width = cols.saturating_sub(4);
    let filled = ((1.0 - progress) * bar_width as f64)
        .round()
        .min(bar_width as f64) as u16;
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

/// Format remaining time as a string.
fn format_time(remaining: Duration) -> String {
    let total_secs = remaining.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

/// Draw the inline timer: time + progress bar on a single line, no screen clear.
pub fn draw_inline_timer(
    stdout: &mut io::Stdout,
    row: u16,
    remaining: Duration,
    progress: f64,
    color: Color,
) -> io::Result<()> {
    let (cols, _) = size()?;
    let time_str = format_time(remaining);

    // Layout: "MM:SS ████░░░░" or "HH:MM:SS ████░░░░"
    let time_width = time_str.len() as u16 + 1; // +1 for space
    let bar_width = cols.saturating_sub(time_width + 2) as usize; // 2 for left margin
    let filled = ((1.0 - progress) * bar_width as f64)
        .round()
        .min(bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);

    let bar = format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty),);

    stdout
        .queue(MoveTo(0, row))?
        .queue(Clear(ClearType::CurrentLine))?
        .queue(SetForegroundColor(color))?
        .queue(SetAttribute(Attribute::Bold))?
        .queue(Print(&time_str))?
        .queue(SetAttribute(Attribute::Reset))?
        .queue(Print(" "))?
        .queue(SetForegroundColor(color))?
        .queue(Print(&bar))?
        .queue(ResetColor)?
        .flush()?;

    Ok(())
}

/// Print finish/cancel message on the line below the timer bar (inline mode).
pub fn print_inline_finish(stdout: &mut io::Stdout, timer_row: u16, msg: &str) -> io::Result<()> {
    stdout
        .queue(MoveTo(0, timer_row))?
        .queue(Print("\r\n"))?
        .queue(Print(msg))?
        .flush()
}

/// Inline interactive prompt: HH:MM:SS entry on the current line.
/// Returns (total_seconds, optional_audio_path, optional_url).
pub fn inline_interactive_prompt() -> io::Result<(u64, Option<PathBuf>, Option<String>)> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(cursor::Show)?;

    let mut digits: [u8; 6] = [0; 6];
    let mut cursor_pos: usize = 0;
    let mut blink_visible = true;
    let mut last_blink = Instant::now();
    let blink_interval = Duration::from_millis(500);
    let mut error_msg: Option<&str> = None;

    // Print a blank line for top padding, then prompt on next line
    stdout.execute(Print("\r\n"))?;
    let (_, prompt_row) = cursor::position()?;

    loop {
        if last_blink.elapsed() >= blink_interval {
            blink_visible = !blink_visible;
            last_blink = Instant::now();
        }

        draw_inline_time_input(
            &mut stdout,
            prompt_row,
            &digits,
            cursor_pos,
            blink_visible,
            error_msg,
        )?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
                if is_quit_event(&key_event) {
                    // Clean up and exit
                    stdout
                        .queue(MoveTo(0, prompt_row))?
                        .queue(Clear(ClearType::CurrentLine))?
                        .flush()?;
                    stdout.execute(cursor::Show)?;
                    terminal::disable_raw_mode()?;
                    std::process::exit(0);
                }
                match key_event.code {
                    KeyCode::Char(c) if c.is_ascii_digit() && cursor_pos < 6 => {
                        let d = c as u8 - b'0';
                        let valid = match cursor_pos {
                            2 | 4 => d <= 5,
                            _ => true,
                        };
                        if valid {
                            digits[cursor_pos] = d;
                            cursor_pos += 1;
                            blink_visible = true;
                            last_blink = Instant::now();
                            error_msg = None;
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
                    KeyCode::Enter => {
                        if digits.iter().all(|&d| d == 0) {
                            error_msg = Some(crate::ERR_ZERO_DURATION);
                        } else {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let hours = (digits[0] as u64) * 10 + (digits[1] as u64);
    let minutes = (digits[2] as u64) * 10 + (digits[3] as u64);
    let seconds = (digits[4] as u64) * 10 + (digits[5] as u64);
    let total_secs = hours * 3600 + minutes * 60 + seconds;

    // Erase prompt line, ask for audio path on same line
    let audio_path = inline_prompt_audio_path(&mut stdout, prompt_row)?;
    let url = inline_prompt_url(&mut stdout, prompt_row)?;

    // Erase the prompt line before returning
    stdout
        .queue(MoveTo(0, prompt_row))?
        .queue(Clear(ClearType::CurrentLine))?
        .flush()?;
    stdout.execute(cursor::Hide)?;
    terminal::disable_raw_mode()?;

    Ok((total_secs, audio_path, url))
}

/// Draw inline HH:MM:SS input on a single line (left-aligned).
fn draw_inline_time_input(
    stdout: &mut io::Stdout,
    row: u16,
    digits: &[u8; 6],
    cursor_pos: usize,
    blink_visible: bool,
    error_msg: Option<&str>,
) -> io::Result<()> {
    stdout
        .queue(MoveTo(0, row))?
        .queue(Clear(ClearType::CurrentLine))?;

    // Label
    stdout
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print("Duration: "))?
        .queue(ResetColor)?;

    for i in 0..6 {
        if i == 2 || i == 4 {
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(":"))?
                .queue(ResetColor)?;
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

    if let Some(err) = error_msg {
        stdout
            .queue(Print("  "))?
            .queue(SetForegroundColor(Color::Red))?
            .queue(Print(err))?;
    }

    stdout.queue(ResetColor)?.flush()?;
    Ok(())
}

/// Inline audio path prompt on a single line.
fn inline_prompt_audio_path(stdout: &mut io::Stdout, row: u16) -> io::Result<Option<PathBuf>> {
    let mut input = String::new();
    let mut error_msg: Option<String> = None;

    loop {
        stdout
            .queue(MoveTo(0, row))?
            .queue(Clear(ClearType::CurrentLine))?
            .queue(SetForegroundColor(Color::DarkGrey))?
            .queue(Print("Audio file (Enter to skip): "))?
            .queue(ResetColor)?
            .queue(SetForegroundColor(Color::Cyan))?
            .queue(SetAttribute(Attribute::Bold))?
            .queue(Print(&input))?
            .queue(SetAttribute(Attribute::Reset))?;

        if input.is_empty() {
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(Print("_"))?;
        }

        if let Some(ref err) = error_msg {
            stdout
                .queue(Print("  "))?
                .queue(SetForegroundColor(Color::Red))?
                .queue(Print(err))?;
        }

        stdout.queue(ResetColor)?.flush()?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
                if is_quit_event(&key_event) {
                    stdout
                        .queue(MoveTo(0, row))?
                        .queue(Clear(ClearType::CurrentLine))?
                        .flush()?;
                    stdout.execute(cursor::Show)?;
                    terminal::disable_raw_mode()?;
                    std::process::exit(0);
                }
                match key_event.code {
                    KeyCode::Enter => {
                        let trimmed = input.trim().trim_matches('"');
                        if trimmed.is_empty() {
                            return Ok(None);
                        }
                        let path = PathBuf::from(trimmed);
                        if path.exists() {
                            return Ok(Some(path));
                        }
                        error_msg = Some(format!("\"{}\" not found", trimmed));
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

/// Interactive prompt: centered HH:MM:SS digit-by-digit entry.
/// Returns (total_seconds, optional_audio_path, optional_url).
pub fn interactive_prompt() -> io::Result<(u64, Option<PathBuf>, Option<String>)> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(cursor::Hide)?;
    stdout.execute(Clear(ClearType::All))?;

    let mut digits: [u8; 6] = [0; 6];
    let mut cursor_pos: usize = 0;
    let mut blink_visible = true;
    let mut last_blink = Instant::now();
    let blink_interval = Duration::from_millis(500);
    let mut error_msg: Option<&str> = None;

    loop {
        if last_blink.elapsed() >= blink_interval {
            blink_visible = !blink_visible;
            last_blink = Instant::now();
        }

        draw_time_input(&mut stdout, &digits, cursor_pos, blink_visible, error_msg)?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
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
                            error_msg = None;
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
                    KeyCode::Enter => {
                        if digits.iter().all(|&d| d == 0) {
                            error_msg = Some(crate::ERR_ZERO_DURATION);
                        } else {
                            break;
                        }
                    }
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
    let url = prompt_url(&mut stdout)?;

    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok((total_secs, audio_path, url))
}

/// Draw the HH:MM:SS input mask centered on screen.
fn draw_time_input(
    stdout: &mut io::Stdout,
    digits: &[u8; 6],
    cursor_pos: usize,
    blink_visible: bool,
    error_msg: Option<&str>,
) -> io::Result<()> {
    let (cols, rows) = size()?;
    let center_row = rows / 2;

    // Error message
    let error_row = center_row.saturating_sub(4);
    stdout
        .queue(MoveTo(0, error_row))?
        .queue(Clear(ClearType::CurrentLine))?;
    if let Some(err) = error_msg {
        let err_col = cols.saturating_sub(err.len() as u16) / 2;
        stdout
            .queue(MoveTo(err_col, error_row))?
            .queue(SetForegroundColor(Color::Red))?
            .queue(SetAttribute(Attribute::Bold))?
            .queue(Print(err))?
            .queue(SetAttribute(Attribute::Reset))?
            .queue(ResetColor)?;
    }

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
            &input[..]
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
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
                if is_quit_event(&key_event) {
                    cleanup_and_exit(stdout);
                }
                match key_event.code {
                    KeyCode::Enter => {
                        let trimmed = input.trim().trim_matches('"');
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

/// Fullscreen URL prompt (centered).
fn prompt_url(stdout: &mut io::Stdout) -> io::Result<Option<String>> {
    stdout.execute(Clear(ClearType::All))?;

    let title = "URL to call when done (optional, press Enter to skip):";
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
            &input[..]
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
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
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
                        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                            stdout.execute(cursor::Hide)?;
                            return Ok(Some(trimmed.to_string()));
                        }
                        error_msg = Some("URL must start with http:// or https://".to_string());
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

/// Inline URL prompt on a single line.
fn inline_prompt_url(stdout: &mut io::Stdout, row: u16) -> io::Result<Option<String>> {
    let mut input = String::new();
    let mut error_msg: Option<String> = None;

    loop {
        stdout
            .queue(MoveTo(0, row))?
            .queue(Clear(ClearType::CurrentLine))?
            .queue(SetForegroundColor(Color::DarkGrey))?
            .queue(Print("URL to call (Enter to skip): "))?
            .queue(ResetColor)?
            .queue(SetForegroundColor(Color::Cyan))?
            .queue(SetAttribute(Attribute::Bold))?
            .queue(Print(&input))?
            .queue(SetAttribute(Attribute::Reset))?;

        if input.is_empty() {
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(Print("_"))?;
        }

        if let Some(ref err) = error_msg {
            stdout
                .queue(Print("  "))?
                .queue(SetForegroundColor(Color::Red))?
                .queue(Print(err))?;
        }

        stdout.queue(ResetColor)?.flush()?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
                if is_quit_event(&key_event) {
                    stdout
                        .queue(MoveTo(0, row))?
                        .queue(Clear(ClearType::CurrentLine))?
                        .flush()?;
                    stdout.execute(cursor::Show)?;
                    terminal::disable_raw_mode()?;
                    std::process::exit(0);
                }
                match key_event.code {
                    KeyCode::Enter => {
                        let trimmed = input.trim();
                        if trimmed.is_empty() {
                            return Ok(None);
                        }
                        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                            return Ok(Some(trimmed.to_string()));
                        }
                        error_msg = Some("Must start with http:// or https://".to_string());
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
