use termion::event::Key;

use crate::editor::Mode;

/// Commands that can be executed by the editor
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Quit the editor
    Quit,
    /// Move the cursor
    Move(isize, isize),
    /// Save the file
    Save,
    /// Toggle mode
    ToggleMode,
    /// Insert a character
    Insert(char),
    /// Delete a character
    Delete(),
}

impl Command {
    /// Parse a command from a byte
    pub fn parse(key: Key, mode: &Mode) -> Result<Self, &'static str> {
        match (mode, key) {
            (Mode::Normal, Key::Char('q')) => Ok(Command::Quit),
            (Mode::Normal, Key::Char('j')) | (Mode::Normal, Key::Down ) => Ok(Command::Move(0, 1)),
            (Mode::Normal, Key::Char('k')) | (Mode::Normal, Key::Up )  => Ok(Command::Move(0, -1)),
            (Mode::Normal, Key::Char('h')) | (Mode::Normal, Key::Left ) => Ok(Command::Move(-1, 0)),
            (Mode::Normal, Key::Char('l')) | (Mode::Normal, Key::Right ) => Ok(Command::Move(1, 0)),
            (Mode::Normal, Key::Char('w')) => Ok(Command::Save),
            (Mode::Normal, Key::Char('i')) => Ok(Command::ToggleMode),
            (Mode::Insert, Key::Esc) => Ok(Command::ToggleMode),
            (Mode::Insert, Key::Char(c)) => Ok(Command::Insert(c.clone())),
            (Mode::Insert, Key::Backspace) => Ok(Command::Delete()),
            (Mode::Insert, Key::Right) => Ok(Command::Move(1, 0)),
            (Mode::Insert, Key::Left) => Ok(Command::Move(-1, 0)),
            (Mode::Insert, Key::Up) => Ok(Command::Move(0, -1)),
            (Mode::Insert, Key::Down) => Ok(Command::Move(0, 1)),
            _ => Err("Invalid command"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_command() {
        assert_eq!(
            Command::parse(Key::Char('q'), &Mode::Normal),
            Ok(Command::Quit)
        );
        assert_eq!(
            Command::parse(Key::Char('j'), &Mode::Normal),
            Ok(Command::Move(0, 1))
        );
        assert_eq!(
            Command::parse(Key::Char('k'), &Mode::Normal),
            Ok(Command::Move(0, -1))
        );
        assert_eq!(
            Command::parse(Key::Char('h'), &Mode::Normal),
            Ok(Command::Move(-1, 0))
        );
        assert_eq!(
            Command::parse(Key::Char('l'), &Mode::Normal),
            Ok(Command::Move(1, 0))
        );
        assert_eq!(
            Command::parse(Key::Char('w'), &Mode::Normal),
            Ok(Command::Save)
        );
        assert_eq!(
            Command::parse(Key::Char('i'), &Mode::Normal),
            Ok(Command::ToggleMode)
        );
        assert_eq!(
            Command::parse(Key::Esc, &Mode::Insert),
            Ok(Command::ToggleMode)
        );
        assert_eq!(
            Command::parse(Key::Char('j'), &Mode::Insert),
            Err("Invalid command")
        );
        assert_eq!(
            Command::parse(Key::Char('k'), &Mode::Insert),
            Err("Invalid command")
        );
        assert_eq!(
            Command::parse(Key::Char('q'), &Mode::Insert),
            Err("Invalid command")
        );
    }
}
