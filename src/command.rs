use termion::event::Key;

/// Commands that can be executed by the editor
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Quit the editor
    Quit,
    /// Move the cursor
    Move(isize, isize),
    /// Save the file
    Save,
}

impl Command {
    /// Parse a command from a byte
    pub fn parse(key: Key) -> Result<Self, &'static str> {
        match key {
            Key::Char('q') => Ok(Command::Quit),
            Key::Char('j') => Ok(Command::Move(0, 1)),
            Key::Char('k') => Ok(Command::Move(0, -1)),
            Key::Char('h') => Ok(Command::Move(-1, 0)),
            Key::Char('l') => Ok(Command::Move(1, 0)),
            Key::Char('w') => Ok(Command::Save),
            _ => Err("Invalid command"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_command() {
        assert_eq!(Command::parse(Key::Char('q')), Ok(Command::Quit));
        assert_eq!(Command::parse(Key::Char('j')), Ok(Command::Move(0, 1)));
        assert_eq!(Command::parse(Key::Char('k')), Ok(Command::Move(0, -1)));
        assert_eq!(Command::parse(Key::Char('h')), Ok(Command::Move(-1, 0)));
        assert_eq!(Command::parse(Key::Char('l')), Ok(Command::Move(1, 0)));
        assert_eq!(Command::parse(Key::Char('w')), Ok(Command::Save));
        assert_eq!(Command::parse(Key::Char('x')), Err("Invalid command"));
    }
}
