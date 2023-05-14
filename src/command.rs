use termion::event::Key;

use crate::editor::Mode;

/// Commands that can be executed by the editor
#[derive(Debug, PartialEq, Clone)]
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
    /// Undo the last insert mode sequence
    Undo,
    /// Redo the last undo
    Redo,
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
            Key::Char('I') => Ok(Command::CommandBlock(vec![
                Command::Move(-isize::MAX, 0),
                Command::ToggleMode,
            ])),
            Key::Char('a') => Ok(Command::CommandBlock(vec![
                Command::Move(1, 0),
                Command::ToggleMode,
            ])),
            Key::Char('A') => Ok(Command::CommandBlock(vec![
                Command::Move(isize::MAX, 0),
                Command::ToggleMode,
            ])),
            Key::Char('o') => Ok(Command::CommandBlock(vec![
                Command::Move(isize::MAX, 0),
                Command::InsertNewLine,
                Command::ToggleMode,
            ])),
            Key::Char('O') => Ok(Command::CommandBlock(vec![
                Command::Move(-isize::MAX, 0),
                Command::InsertNewLine,
                Command::Move(0, -1),
                Command::ToggleMode,
            ])),
            // Undo and redo
            Key::Char('u') => Ok(Command::Undo),
            Key::Char('r') => Ok(Command::Redo),
            // Quit
            Key::Char('q') => Ok(Command::Quit),
            // Move
            Key::Char('j') | Key::Down => Ok(Command::Move(0, 1)),
            Key::Char('k') | Key::Up => Ok(Command::Move(0, -1)),
            Key::Char('h') | Key::Left => Ok(Command::Move(-1, 0)),
            Key::Char('l') | Key::Right => Ok(Command::Move(1, 0)),
            Key::Char('$') => Ok(Command::Move(isize::MAX, 0)),
            Key::Char('0') => Ok(Command::Move(-isize::MAX, 0)),
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

    /// Flatten nested Command::CommandBlock by one level
    pub fn flatten(&self) -> Self {
        // Auxiliary function to flatten into a vector of commands
        fn flatten_vec(cmd: Command) -> Vec<Command> {
            match cmd {
                Command::CommandBlock(cmds) => cmds
                    .into_iter()
                    .flat_map(|cmd| flatten_vec(cmd))
                    .collect::<Vec<Command>>(),
                cmd => vec![cmd],
            }
        }

        let flat_cmd = flatten_vec(self.clone());

        Command::CommandBlock(flat_cmd)
    }

    /// Filter commands in a Command::CommandBlock
    pub fn filter(&self, f: fn(&Command) -> bool) -> Self {
        match self {
            Command::CommandBlock(cmds) => {
                let filtered_cmds = cmds
                    .into_iter()
                    .filter(|cmd| f(cmd))
                    .cloned()
                    .collect::<Vec<Command>>();

                Command::CommandBlock(filtered_cmds)
            }
            cmd => cmd.clone(),
        }
    }

    /// Reverse commands in a Command::CommandBlock
    pub fn rev(&self) -> Self {
        match self {
            Command::CommandBlock(cmds) => {
                let mut rev_cmds = cmds.clone();
                rev_cmds.reverse();

                Command::CommandBlock(rev_cmds)
            }
            cmd => cmd.clone(),
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
            Command::parse(Key::Char('✨'), &Mode::Normal),
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

    #[test]
    fn command_flatten_command_blocks() {
        assert_eq!(
            Command::CommandBlock(vec![
                Command::CommandBlock(vec![Command::Insert('a')]),
                Command::CommandBlock(vec![Command::Insert('b')]),
                Command::CommandBlock(vec![Command::Insert('c')]),
            ])
            .flatten(),
            Command::CommandBlock(vec![
                Command::Insert('a'),
                Command::Insert('b'),
                Command::Insert('c'),
            ])
        );

        assert_eq!(
            Command::CommandBlock(vec![
                Command::CommandBlock(vec![Command::Insert('a')]),
                Command::CommandBlock(vec![Command::Insert('b')]),
                Command::CommandBlock(vec![Command::Insert('c')]),
                Command::Insert('d'),
            ])
            .flatten(),
            Command::CommandBlock(vec![
                Command::Insert('a'),
                Command::Insert('b'),
                Command::Insert('c'),
                Command::Insert('d'),
            ])
        );

        assert_eq!(
            Command::ToggleMode.flatten(),
            Command::CommandBlock(vec![Command::ToggleMode])
        )
    }

    #[test]
    fn command_filter_command_blocks() {
        assert_eq!(
            Command::CommandBlock(vec![
                Command::Insert('a'),
                Command::ToggleMode,
                Command::Move(10, -1),
            ])
            .filter(|cmd| match cmd {
                Command::Insert(_) => true,
                _ => false,
            }),
            Command::CommandBlock(vec![Command::Insert('a'),])
        );

        assert_eq!(
            Command::CommandBlock(vec![
                Command::CommandBlock(vec![Command::Insert('a')]),
                Command::CommandBlock(vec![Command::Insert('b')]),
                Command::CommandBlock(vec![Command::Insert('c')]),
            ])
            .filter(|cmd| match cmd {
                Command::CommandBlock(_) => true,
                _ => false,
            }),
            Command::CommandBlock(vec![
                Command::CommandBlock(vec![Command::Insert('a')]),
                Command::CommandBlock(vec![Command::Insert('b')]),
                Command::CommandBlock(vec![Command::Insert('c')]),
            ])
        );
    }

    #[test]
    fn command_reverse() {
        assert_eq!(
            Command::CommandBlock(vec![
                Command::Insert('a'),
                Command::Insert('b'),
                Command::Insert('c'),
            ])
            .rev(),
            Command::CommandBlock(vec![
                Command::Insert('c'),
                Command::Insert('b'),
                Command::Insert('a'),
            ])
        );
    }
}
