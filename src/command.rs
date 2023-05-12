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
    Delete,
    /// Insert a new line
    InsertNewLine,
    /// CommandBlock
    CommandBlock(Vec<Command>),
}

impl Command {
    /// Parse a command from a termion::event::Key object
    pub fn parse(key: Key, mode: &Mode) -> Result<Self, &'static str> {
        match mode {
            Mode::Normal => Self::parse_normal_mode(key),
            Mode::Insert => Self::parse_insert_mode(key),
        }
    }

    /// Parse a command in normal mode from a termion::event::Key object
    fn parse_normal_mode(key: Key) -> Result<Self, &'static str> {
        match key {
            // Go to insert mode
            Key::Char('i') => Ok(Command::ToggleMode),
            // Quit
            Key::Char('q') => Ok(Command::Quit),
            // Move
            Key::Char('j') | Key::Down => Ok(Command::Move(0, 1)),
            Key::Char('k') | Key::Up => Ok(Command::Move(0, -1)),
            Key::Char('h') | Key::Left => Ok(Command::Move(-1, 0)),
            Key::Char('l') | Key::Right => Ok(Command::Move(1, 0)),
            // Save
            Key::Char('w') => Ok(Command::Save),
            _ => Err("Invalid command"),
        }
    }

    /// Parse a command in insert mode from a termion::event::Key object
    fn parse_insert_mode(key: Key) -> Result<Self, &'static str> {
        match key {
            // Go to normal mode
            Key::Esc => Ok(Command::ToggleMode),
            // Insert a character
            Key::Char(c) => Self::parse_insert_mode_char(c),
            // Delete a character
            Key::Backspace => Ok(Command::Delete),
            // Move
            Key::Right => Ok(Command::Move(1, 0)),
            Key::Left => Ok(Command::Move(-1, 0)),
            Key::Up => Ok(Command::Move(0, -1)),
            Key::Down => Ok(Command::Move(0, 1)),
            _ => Err("Invalid command"),
        }
    }

    /// Parse a character in insert mode
    fn parse_insert_mode_char(c: char) -> Result<Self, &'static str> {
        match c {
            // Insert new line
            '\n' => Ok(Self::InsertNewLine),
            // Insert a tab (4 spaces for now)
            '\t' => Ok(Command::CommandBlock(vec![
                Command::Insert(' '),
                Command::Insert(' '),
                Command::Insert(' '),
                Command::Insert(' '),
            ])),
            // Insert another character
            _ => Ok(Command::Insert(c)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_normal_mode() {
        assert_eq!(
            Command::parse(Key::Char('q'), &Mode::Normal),
            Ok(Command::Quit)
        );
        assert_eq!(
            Command::parse(Key::Char('j'), &Mode::Normal),
            Ok(Command::Move(0, 1))
        );
        assert_eq!(
            Command::parse(Key::Down, &Mode::Normal),
            Ok(Command::Move(0, 1))
        );
        assert_eq!(
            Command::parse(Key::Char('k'), &Mode::Normal),
            Ok(Command::Move(0, -1))
        );
        assert_eq!(
            Command::parse(Key::Up, &Mode::Normal),
            Ok(Command::Move(0, -1))
        );
        assert_eq!(
            Command::parse(Key::Char('h'), &Mode::Normal),
            Ok(Command::Move(-1, 0))
        );
        assert_eq!(
            Command::parse(Key::Left, &Mode::Normal),
            Ok(Command::Move(-1, 0))
        );
        assert_eq!(
            Command::parse(Key::Char('l'), &Mode::Normal),
            Ok(Command::Move(1, 0))
        );
        assert_eq!(
            Command::parse(Key::Right, &Mode::Normal),
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
    }

    #[test]
    fn parse_insert_mode() {
        assert_eq!(
            Command::parse(Key::Esc, &Mode::Insert),
            Ok(Command::ToggleMode)
        );
        assert_eq!(
            Command::parse(Key::Char('j'), &Mode::Insert),
            Ok(Command::Insert('j'))
        );
        assert_eq!(
            Command::parse(Key::Char('k'), &Mode::Insert),
            Ok(Command::Insert('k'))
        );
        assert_eq!(
            Command::parse(Key::Char('q'), &Mode::Insert),
            Ok(Command::Insert('q'))
        );
        assert_eq!(
            Command::parse(Key::Backspace, &Mode::Insert),
            Ok(Command::Delete)
        );
        assert_eq!(
            Command::parse(Key::Right, &Mode::Insert),
            Ok(Command::Move(1, 0))
        );
        assert_eq!(
            Command::parse(Key::Left, &Mode::Insert),
            Ok(Command::Move(-1, 0))
        );
        assert_eq!(
            Command::parse(Key::Up, &Mode::Insert),
            Ok(Command::Move(0, -1))
        );
        assert_eq!(
            Command::parse(Key::Down, &Mode::Insert),
            Ok(Command::Move(0, 1))
        );
    }

    #[test]
    fn parse_invalid_command() {
        assert_eq!(
            Command::parse(Key::Char('a'), &Mode::Normal),
            Err("Invalid command")
        );
        assert_eq!(
            Command::parse(Key::Null, &Mode::Insert),
            Err("Invalid command")
        );
    }

    #[test]
    fn parse_insert_mode_char() {
        assert_eq!(
            Command::parse_insert_mode_char('\n'),
            Ok(Command::InsertNewLine)
        );
        assert_eq!(
            Command::parse_insert_mode_char('\t'),
            Ok(Command::CommandBlock(vec![
                Command::Insert(' '),
                Command::Insert(' '),
                Command::Insert(' '),
                Command::Insert(' '),
            ]))
        );
        assert_eq!(
            Command::parse_insert_mode_char('a'),
            Ok(Command::Insert('a'))
        );
        assert_eq!(
            Command::parse_insert_mode_char('à'),
            Ok(Command::Insert('à'))
        );
    }
}