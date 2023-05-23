use std::{collections::HashSet, fmt::Display, process::exit};

use crate::{
    command::Command,
    file::File,
    terminal::{termion::TermionTerminalDrawer, StatusBarInfos, TerminalDrawer},
    view::View,
};
use termion::input::TermRead;

/// Editor structure
/// represents the state of the program
pub struct Editor {
    /// The path of the file being edited
    file_path: String,
    /// The name of the file being edited
    file_name: String,
    /// The current view of the file
    view: View,
    /// The Tui responsible for drawing the editor
    tui: Box<dyn TerminalDrawer>,
    /// The mode of the editor
    mode: Mode,
}

#[derive(Clone)]
/// Mode of the editor
pub enum Mode {
    /// Normal mode
    Normal,
    /// Insert mode
    Insert,
    /// Rename mode
    Rename,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode = match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Rename => "RENAME",
        };
        write!(f, "{}", mode)
    }
}

pub enum RefreshOrder {
    /// No need to refresh the screen
    None,
    /// Refresh the cursor position
    CursorPos,
    /// Refresh the given lines
    Lines(HashSet<usize>),
    /// Refresh the status bar
    StatusBar,
    /// Refresh the whole screen
    AllLines,
}

impl Editor {
    /// Create a new editor
    pub fn new(file_name: &str) -> Self {
        Self {
            file_path: "./".to_string(),
            file_name: file_name.to_string(),
            view: View::new(File::new(), 0, 0),
            tui: TermionTerminalDrawer::new(),
            mode: Mode::Normal,
        }
    }

    /// Open a file in the editor
    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let content = File::from_string(&content);
        let view = View::new(content, 0, 0);

        let (path, file_name) = Self::split_path_name(path);

        Ok(Self {
            file_path: path.to_string(),
            file_name: file_name.to_string(),
            view,
            tui: TermionTerminalDrawer::new(),
            mode: Mode::Normal,
        })
    }

    fn split_path_name(path: &str) -> (String, String) {
        // if there is no '/' in the path, the file is in the current directory
        if !path.contains('/') {
            return ("./".to_string(), path.to_string());
        }
        let mut path = path.to_string();
        let mut file_name = path.clone();
        let mut i = path.len() - 1;
        while i > 0 {
            if path.chars().nth(i).unwrap() == '/' {
                file_name = path.split_off(i + 1);
                break;
            }
            i -= 1;
        }
        (path, file_name)
    }

    /// Close the editor
    fn terminate(&mut self) {
        self.tui.terminate();
        exit(0);
    }

    /// Save the current file
    fn save(&self) {
        if self.file_name == "" {
            return;
        }
        let path = String::from(&self.file_path) + &self.file_name;
        let content = self.view.dump_file();
        std::fs::write(path.clone() + ".tmp", content).unwrap_or_default();
        std::fs::rename(path.clone() + ".tmp", path).unwrap_or_default();
    }

    /// Rename the current file
    fn rename(&mut self, c: Option<char>) {
        match c {
            None => {
                // delete a char
                self.file_name.pop();
            }
            Some(c) => match c {
                ' ' | '\'' => self.file_name = self.file_name.clone() + "_",
                _ => self.file_name = self.file_name.clone() + &c.to_string(),
            },
        }
    }

    /// Make status bar infos
    fn get_status_bar_infos(&self) -> StatusBarInfos {
        StatusBarInfos {
            file_path: self.file_path.clone(),
            file_name: self.file_name.clone(),
            mode: self.mode.clone(),
        }
    }

    /// Execute an editor command
    /// - Quit: exit the program
    /// - Move: move the cursor
    /// - Save: save the file
    /// - Rename: rename the file
    /// - ToggleMode: toogle editor mode
    /// - Insert: insert a character
    /// - Delete: delete a character
    fn execute(&mut self, cmd: Command) -> RefreshOrder {
        match cmd {
            Command::Quit => {
                self.terminate();
                // Doesn't matter as self.terminate() never returns
                RefreshOrder::AllLines
            }
            Command::Move(x, y) => {
                let scroll = self.view.navigate(x, y);
                if scroll {
                    RefreshOrder::AllLines
                } else {
                    RefreshOrder::CursorPos
                }
            }
            Command::Save => {
                self.save();
                RefreshOrder::StatusBar
            }
            Command::Rename(c) => {
                self.rename(c);
                RefreshOrder::StatusBar
            }
            Command::ToggleMode => {
                self.toggle_mode();
                RefreshOrder::StatusBar
            }
            Command::ToggleRename => {
                self.togle_rename();
                RefreshOrder::StatusBar
            }
            Command::Insert(c) => {
                let scroll = self.view.insert(c);
                if scroll {
                    RefreshOrder::AllLines
                } else {
                    // Refresh only the current line: self.view.cursor.1
                    RefreshOrder::Lines(HashSet::from_iter(vec![self.view.cursor.1]))
                }
            }
            Command::InsertNewLine => {
                let scroll = self.view.insert_new_line();
                if scroll {
                    RefreshOrder::AllLines
                } else {
                    // Refresh only the lines below the cursor
                    RefreshOrder::Lines(HashSet::from_iter(
                        self.view.cursor.1 - 1..self.view.height,
                    ))
                }
            }
            Command::Delete => {
                let scroll = self.view.delete();
                if scroll {
                    // If we scroll (because we deleted a char at the left of the view),
                    // we need to refresh all lines.
                    RefreshOrder::AllLines
                } else {
                    // Refresh only the lines below the cursor
                    RefreshOrder::Lines(HashSet::from_iter(self.view.cursor.1..self.view.height))
                }
            }
            Command::CommandBlock(cmds) => {
                cmds.into_iter().fold(RefreshOrder::None, |refr, cmd| {
                    use RefreshOrder::*;
                    match (refr, self.execute(cmd)) {
                        (None, r) | (r, None) => r,
                        (Lines(mut s1), Lines(s2)) => {
                            s1.extend(s2);
                            Lines(s1)
                        }
                        _ => AllLines,
                    }
                })
            }
        }
    }

    /// Toggle the mode of the editor between normal and insert
    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Normal => Mode::Insert,
            Mode::Insert => Mode::Normal,
            Mode::Rename => Mode::Normal,
        }
    }

    fn togle_rename(&mut self) {
        self.mode = match self.mode {
            Mode::Normal => Mode::Rename,
            Mode::Rename => Mode::Normal,
            _ => Mode::Normal, // Could not be in insert mode
        }
    }

    /// Run the editor loop
    pub fn run(&mut self) {
        // set view size
        let (width, height) = self.tui.get_term_size();

        self.view.resize(height, width);
        // draw initial view
        self.tui.clear();
        self.tui.draw(&self.view, &self.get_status_bar_infos());

        let stdin = std::io::stdin().keys();

        for c in stdin {
            if let Ok(c) = c {
                if let Ok(cmd) = Command::parse(c, &self.mode) {
                    match self.execute(cmd) {
                        RefreshOrder::None => (),
                        RefreshOrder::CursorPos => self.tui.move_cursor(self.view.cursor),
                        RefreshOrder::StatusBar => {
                            self.tui.draw_status_bar(&self.get_status_bar_infos());
                            self.tui.move_cursor(self.view.cursor)
                        }
                        RefreshOrder::Lines(lines) => self.tui.draw_lines(&self.view, lines),
                        RefreshOrder::AllLines => {
                            self.tui.draw(&self.view, &self.get_status_bar_infos())
                        }
                    }
                }
            }
        }
    }
}
