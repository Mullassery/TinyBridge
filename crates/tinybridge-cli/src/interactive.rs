//! Interactive REPL mode for the CLI -- built but not yet wired into any
//! subcommand (verified: nothing outside this module and its own code
//! constructs `InteractiveMode`, so it's unreachable dead code today, same
//! situation as `tinybridge-core`'s `windows_adapter.rs`). Kept as
//! scaffolding for a future `tinybridge interactive`/`tinybridge repl`
//! subcommand rather than deleted.
#![allow(dead_code)]

use crate::keyboard::{KeyAction, KeyboardHandler};
use crate::output;
use std::io::{self, Write};

/// Interactive REPL mode for TinyBridge CLI
pub struct InteractiveMode {
    history: Vec<String>,
    history_index: Option<usize>,
}

impl InteractiveMode {
    pub fn new() -> Self {
        InteractiveMode {
            history: Vec::new(),
            history_index: None,
        }
    }

    /// Run interactive mode (REPL)
    pub async fn run(&mut self) -> io::Result<()> {
        output::print_header("TinyBridge Interactive Mode");
        println!("Type 'help' for commands or 'exit' to quit\n");

        let mut input = String::new();
        let mut cursor_pos;

        loop {
            // Print prompt
            print!("tinybridge> ");
            io::stdout().flush()?;

            // Clear input for next iteration
            input.clear();
            cursor_pos = 0;

            // Read line with full keyboard support
            if !self.read_line_with_keyboard(&mut input, &mut cursor_pos)? {
                break; // Exit on Ctrl-D
            }

            // Process command
            let trimmed = input.trim();
            if !trimmed.is_empty() {
                self.history.push(trimmed.to_string());
                self.process_command(trimmed).await;
            }

            self.history_index = None;
        }

        println!("\nGoodbye!");
        Ok(())
    }

    /// Read a line with full keyboard support
    fn read_line_with_keyboard(
        &self,
        input: &mut String,
        cursor_pos: &mut usize,
    ) -> io::Result<bool> {
        loop {
            // Poll keyboard with 100ms timeout
            match KeyboardHandler::poll_event(100) {
                Ok(Some(action)) => {
                    match action {
                        KeyAction::Exit => {
                            return Ok(false); // Exit REPL
                        }
                        KeyAction::Interrupt => {
                            println!("^C");
                            *input = String::new();
                            *cursor_pos = 0;
                            return Ok(true); // Cancel current line, continue
                        }
                        KeyAction::Enter => {
                            println!(); // Newline after input
                            return Ok(true); // Submit line
                        }
                        KeyAction::ClearScreen => {
                            // Clear screen and reprint prompt
                            print!("\x1B[2J\x1B[H"); // ANSI clear screen
                            print!("tinybridge> ");
                            io::stdout().flush()?;
                        }
                        _ => {
                            // Handle other key actions
                            KeyboardHandler::handle_action(action, input, cursor_pos);

                            // Redraw input line
                            self.redraw_input_line(input, *cursor_pos)?;
                        }
                    }
                }
                Ok(None) => {
                    // Timeout, continue polling
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    /// Redraw the input line with cursor
    fn redraw_input_line(&self, input: &str, cursor_pos: usize) -> io::Result<()> {
        // Move cursor to start of input (after prompt)
        print!("\r\x1B[K"); // Clear line
        print!("tinybridge> ");

        let formatted = KeyboardHandler::format_input_with_cursor(input, cursor_pos);
        print!("{}", formatted);

        io::stdout().flush()?;
        Ok(())
    }

    /// Process a command
    async fn process_command(&mut self, cmd: &str) {
        match cmd {
            "help" => self.print_help(),
            "exit" | "quit" => {
                println!("Exiting...");
            }
            "clear" => {
                print!("\x1B[2J\x1B[H"); // ANSI clear screen
                let _ = io::stdout().flush();
            }
            "history" => self.print_history(),
            "keyboard-help" => {
                println!("{}", KeyboardHandler::get_keyboard_help());
            }
            "" => {} // Empty command
            _ => {
                println!(
                    "Unknown command: '{}'. Type 'help' for available commands.",
                    cmd
                );
            }
        }
    }

    fn print_help(&self) {
        println!(
            r#"
Available Commands:

  help              Show this help message
  exit, quit        Exit interactive mode
  clear             Clear the screen
  history           Show command history
  keyboard-help     Show keyboard shortcuts

Examples:
  tinybridge> launch rust
  tinybridge> doctor
  tinybridge> status

Press Ctrl-D to exit, Ctrl-C to cancel current line.
"#
        );
    }

    fn print_history(&self) {
        if self.history.is_empty() {
            println!("No command history");
        } else {
            println!("Command History:");
            for (i, cmd) in self.history.iter().enumerate() {
                println!("  {}: {}", i + 1, cmd);
            }
        }
    }
}

impl Default for InteractiveMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_mode_creation() {
        let mode = InteractiveMode::new();
        assert!(mode.history.is_empty());
        assert_eq!(mode.history_index, None);
    }

    #[test]
    fn test_history_management() {
        let mut mode = InteractiveMode::new();
        mode.history.push("launch rust".to_string());
        mode.history.push("doctor".to_string());

        assert_eq!(mode.history.len(), 2);
    }

    #[test]
    fn test_default_creation() {
        let _mode = InteractiveMode::default();
    }
}
