use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Keyboard event handler for interactive CLI mode
pub struct KeyboardHandler;

/// Keyboard shortcuts and their actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Ctrl-C: Interrupt current operation
    Interrupt,
    /// Ctrl-D: Exit CLI
    Exit,
    /// Ctrl-L: Clear screen
    ClearScreen,
    /// Ctrl-U: Clear line
    ClearLine,
    /// Ctrl-W: Clear word
    ClearWord,
    /// Ctrl-A: Go to start of line
    StartOfLine,
    /// Ctrl-E: Go to end of line
    EndOfLine,
    /// Ctrl-K: Kill to end of line
    KillToEndOfLine,
    /// Ctrl-H: Backspace
    Backspace,
    /// Tab: Auto-complete
    AutoComplete,
    /// Enter: Submit
    Enter,
    /// Arrow keys: Navigation
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    /// Regular character
    Char(char),
    /// No action
    None,
}

impl KeyboardHandler {
    /// Poll for keyboard event with timeout
    pub fn poll_event(timeout_ms: u64) -> std::io::Result<Option<KeyAction>> {
        if event::poll(Duration::from_millis(timeout_ms))? {
            if let Event::Key(key_event) = event::read()? {
                return Ok(Some(Self::map_key_event(key_event)));
            }
        }
        Ok(None)
    }

    /// Map crossterm KeyEvent to KeyAction
    fn map_key_event(key_event: KeyEvent) -> KeyAction {
        match key_event.code {
            KeyCode::Char(c) => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => KeyAction::Interrupt,
                        'd' => KeyAction::Exit,
                        'l' => KeyAction::ClearScreen,
                        'u' => KeyAction::ClearLine,
                        'w' => KeyAction::ClearWord,
                        'a' => KeyAction::StartOfLine,
                        'e' => KeyAction::EndOfLine,
                        'k' => KeyAction::KillToEndOfLine,
                        'h' => KeyAction::Backspace,
                        _ => KeyAction::Char(c),
                    }
                } else if key_event.modifiers.contains(KeyModifiers::ALT) {
                    // Alt shortcuts (reserved for future use)
                    KeyAction::Char(c)
                } else {
                    // Regular character
                    KeyAction::Char(c)
                }
            }
            KeyCode::Tab => KeyAction::AutoComplete,
            KeyCode::Enter => KeyAction::Enter,
            KeyCode::Backspace => KeyAction::Backspace,
            KeyCode::Up => KeyAction::ArrowUp,
            KeyCode::Down => KeyAction::ArrowDown,
            KeyCode::Left => KeyAction::ArrowLeft,
            KeyCode::Right => KeyAction::ArrowRight,
            _ => KeyAction::None,
        }
    }

    /// Handle keyboard action with user input buffer
    pub fn handle_action(action: KeyAction, input: &mut String, cursor_pos: &mut usize) {
        match action {
            KeyAction::Char(c) => {
                input.insert(*cursor_pos, c);
                *cursor_pos += 1;
            }
            KeyAction::Backspace => {
                if *cursor_pos > 0 {
                    input.remove(*cursor_pos - 1);
                    *cursor_pos -= 1;
                }
            }
            KeyAction::Enter => {
                // Submit input (handled by caller)
            }
            KeyAction::ArrowLeft => {
                if *cursor_pos > 0 {
                    *cursor_pos -= 1;
                }
            }
            KeyAction::ArrowRight => {
                if *cursor_pos < input.len() {
                    *cursor_pos += 1;
                }
            }
            KeyAction::StartOfLine => {
                *cursor_pos = 0;
            }
            KeyAction::EndOfLine => {
                *cursor_pos = input.len();
            }
            KeyAction::ClearLine => {
                input.clear();
                *cursor_pos = 0;
            }
            KeyAction::ClearWord => {
                if *cursor_pos > 0 {
                    let start = input[..*cursor_pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
                    input.drain(start..*cursor_pos);
                    *cursor_pos = start;
                }
            }
            KeyAction::KillToEndOfLine => {
                input.truncate(*cursor_pos);
            }
            _ => {}
        }
    }

    /// Format input with cursor for display
    pub fn format_input_with_cursor(input: &str, cursor_pos: usize) -> String {
        if cursor_pos <= input.len() {
            let (before, after) = input.split_at(cursor_pos);
            format!("{}\u{2590}{}", before, after) // Unicode right-half block cursor
        } else {
            format!("{}\u{2590}", input)
        }
    }

    /// Get help text for keyboard shortcuts
    pub fn get_keyboard_help() -> &'static str {
        r#"
Keyboard Shortcuts:

Navigation:
  Ctrl-A    Go to start of line
  Ctrl-E    Go to end of line
  Ctrl-Left Move word left
  Ctrl-Right Move word right
  Arrow Keys Navigate input

Editing:
  Ctrl-H    Backspace
  Ctrl-U    Clear line
  Ctrl-W    Clear word
  Ctrl-K    Kill to end of line
  Tab       Auto-complete

Control:
  Enter     Submit command
  Ctrl-C    Interrupt operation
  Ctrl-L    Clear screen
  Ctrl-D    Exit CLI
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_action_mapping() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let action = KeyboardHandler::map_key_event(key);
        assert_eq!(action, KeyAction::Interrupt);
    }

    #[test]
    fn test_character_input() {
        let mut input = String::new();
        let mut cursor = 0;

        KeyboardHandler::handle_action(KeyAction::Char('h'), &mut input, &mut cursor);
        KeyboardHandler::handle_action(KeyAction::Char('i'), &mut input, &mut cursor);

        assert_eq!(input, "hi");
        assert_eq!(cursor, 2);
    }

    #[test]
    fn test_backspace() {
        let mut input = String::from("hello");
        let mut cursor = 5;

        KeyboardHandler::handle_action(KeyAction::Backspace, &mut input, &mut cursor);

        assert_eq!(input, "hell");
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = String::from("hello");
        let mut cursor = 5;

        KeyboardHandler::handle_action(KeyAction::ArrowLeft, &mut input, &mut cursor);
        assert_eq!(cursor, 4);

        KeyboardHandler::handle_action(KeyAction::StartOfLine, &mut input, &mut cursor);
        assert_eq!(cursor, 0);

        KeyboardHandler::handle_action(KeyAction::EndOfLine, &mut input, &mut cursor);
        assert_eq!(cursor, 5);
    }

    #[test]
    fn test_clear_word() {
        let mut input = String::from("hello world");
        let mut cursor = 11;

        KeyboardHandler::handle_action(KeyAction::ClearWord, &mut input, &mut cursor);

        assert_eq!(input, "hello ");
        assert_eq!(cursor, 6);
    }

    #[test]
    fn test_input_with_cursor_display() {
        let input = "hello";
        let formatted = KeyboardHandler::format_input_with_cursor(input, 3);

        assert!(formatted.contains("\u{2590}")); // Contains cursor character
        assert!(formatted.starts_with("hel"));
    }

    #[test]
    fn test_clear_line() {
        let mut input = String::from("test input");
        let mut cursor = 10;

        KeyboardHandler::handle_action(KeyAction::ClearLine, &mut input, &mut cursor);

        assert_eq!(input, "");
        assert_eq!(cursor, 0);
    }
}
