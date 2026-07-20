use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::os::unix::net::UnixStream;

pub struct TerminalHandler {
    raw_mode_enabled: bool,
}

impl TerminalHandler {
    pub fn new() -> Result<Self> {
        Ok(TerminalHandler {
            raw_mode_enabled: false,
        })
    }

    pub fn enable_raw_mode(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        self.raw_mode_enabled = true;
        Ok(())
    }

    pub fn disable_raw_mode(&mut self) -> Result<()> {
        if self.raw_mode_enabled {
            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
            disable_raw_mode()?;
            self.raw_mode_enabled = false;
        }
        Ok(())
    }

    pub async fn handle_interactive_shell(&mut self, socket: UnixStream) -> Result<()> {
        self.enable_raw_mode()?;

        let result = self.process_keyboard_and_output(socket).await;

        let _ = self.disable_raw_mode();

        result
    }

    async fn process_keyboard_and_output(&mut self, mut socket: UnixStream) -> Result<()> {
        loop {
            // Check for keyboard input with timeout
            if event::poll(std::time::Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key_event) => {
                        let should_exit = self.handle_key_event(key_event, &mut socket)?;
                        if should_exit {
                            break;
                        }
                    }
                    Event::Resize(_cols, _rows) => {
                        // Handle resize - would notify daemon about new size
                    }
                    _ => {
                        // Ignore other events
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_key_event(&self, key: KeyEvent, socket: &mut UnixStream) -> Result<bool> {
        let bytes = match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => return Ok(true), // Exit on Ctrl+C
                        'd' => {
                            // Send EOF on Ctrl+D
                            b"\x04".to_vec()
                        }
                        _ => {
                            // Send Ctrl+char
                            vec![c as u8 & 0x1f]
                        }
                    }
                } else if key.modifiers.contains(KeyModifiers::ALT) {
                    // Send ESC + char for Alt+key combinations
                    let mut v = vec![27]; // ESC
                    v.push(c as u8);
                    v
                } else {
                    // Regular character
                    c.to_string().into_bytes()
                }
            }
            KeyCode::Enter => b"\r".to_vec(),
            KeyCode::Tab => b"\t".to_vec(),
            KeyCode::Backspace => b"\x08".to_vec(),
            KeyCode::Delete => b"\x7f".to_vec(),
            KeyCode::Esc => b"\x1b".to_vec(),
            KeyCode::Up => b"\x1b[A".to_vec(),    // Arrow up
            KeyCode::Down => b"\x1b[B".to_vec(),  // Arrow down
            KeyCode::Right => b"\x1b[C".to_vec(), // Arrow right
            KeyCode::Left => b"\x1b[D".to_vec(),  // Arrow left
            KeyCode::Home => b"\x1b[H".to_vec(),
            KeyCode::End => b"\x1b[F".to_vec(),
            KeyCode::PageUp => b"\x1b[5~".to_vec(),
            KeyCode::PageDown => b"\x1b[6~".to_vec(),
            KeyCode::F(n) => {
                // F1-F12 keys
                match n {
                    1 => b"\x1bOP".to_vec(),
                    2 => b"\x1bOQ".to_vec(),
                    3 => b"\x1bOR".to_vec(),
                    4 => b"\x1bOS".to_vec(),
                    5..=8 => format!("\x1b[{};2~", 14 + (n - 5) * 2).into_bytes(),
                    9..=12 => format!("\x1b[{};2~", 23 + (n - 9)).into_bytes(),
                    _ => return Ok(false), // Unknown F key
                }
            }
            _ => return Ok(false), // Ignore other keys
        };

        // Send to daemon
        socket.write_all(&bytes)?;
        socket.flush()?;

        Ok(false)
    }
}

impl Drop for TerminalHandler {
    fn drop(&mut self) {
        let _ = self.disable_raw_mode();
    }
}
