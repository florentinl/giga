use std::{collections::HashSet, process::exit};

use crate::{
    command::Command,
    file::File,
    tui::{StatusBar, Tui},
    view::View,
};
use termion::input::TermRead;

/// Editor structure
/// represents the state of the program
pub struct Editor {
    /// The path of the file being edited
    path: String,
    /// The name of the file being edited
    file_name: String,
    /// The current view of the file
    view: View,
    /// The Tui responsible for drawing the editor
    tui: Tui,
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

pub enum RefreshOrder {
    /// No need to refresh the screen
    None,
    /// Refresh the cursor position
    CursorPos,
    /// Refresh the given lines
    Lines(HashSet<u16>),
    /// Refresh the status bar
    StatusBar,
    /// Refresh the whole screen
    AllLines,
}

impl Editor {
    /// Create a new editor
    pub fn new(file_name: &str) -> Self {
        Self {
            path: "./".to_string(),
            file_name: file_name.to_string(),
            view: View::new(File::new(), 0, 0),
            tui: Tui::new(),
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
            path: path.to_string(),
            file_name: file_name.to_string(),
            view,
            tui: Tui::new(),
            mode: Mode::Normal,
        })
    }

    fn split_path_name(path: &str) -> (String, String) {
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
        self.tui.cleanup();
        exit(0);
    }

    /// Save the current file
    fn save(&self) {
        if self.file_name == "" {
            return;
        }
        let path = String::from(&self.path) + &self.file_name;
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
                    return RefreshOrder::AllLines;
                } else {
                    // Refresh only the current line: self.view.cursor.1
                    RefreshOrder::Lines(HashSet::from_iter(vec![self.view.cursor.1 as u16]))
                }
            }
            Command::InsertNewLine => {
                let scroll = self.view.insert_new_line();
                if scroll {
                    // If we scroll (because we are at the bottom of the view),
                    // we need to refresh all lines.
                    RefreshOrder::AllLines
                } else {
                    // Refresh only the lines below the cursor
                    RefreshOrder::Lines(HashSet::from_iter(
                        self.view.cursor.1 as u16..self.view.height as u16,
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
                    RefreshOrder::Lines(HashSet::from_iter(
                        self.view.cursor.1 as u16..self.view.height as u16,
                    ))
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
        let mut sb = StatusBar {
            path: self.path.clone(),
            file_name: self.file_name.clone(),
            mode: self.mode.clone(),
        };

        // set view size
        let (width, height) = self.tui.get_term_size();

        // height - 1 to leave space for the status bar
        // width - 4 to leave space for the line numbers
        self.view
            .resize((height - 1) as usize, (width - 4) as usize);
        // draw initial view
        self.tui.clear();
        self.tui.draw_view(&self.view, &sb);

        let stdin = std::io::stdin().keys();

        for c in stdin {
            if let Ok(c) = c {
                if let Ok(cmd) = Command::parse(c, &self.mode) {
                    match self.execute(cmd) {
                        RefreshOrder::None => (),
                        RefreshOrder::CursorPos => {
                            let (x, y) = self.view.cursor;
                            self.tui.move_cursor(x, y)
                        }
                        RefreshOrder::StatusBar => {
                            sb.file_name = self.file_name.clone();
                            sb.mode = self.mode.clone();
                            self.tui.draw_status_bar(&sb, height, width)
                        }
                        RefreshOrder::Lines(lines) => self.tui.refresh_lines(&self.view, lines),
                        RefreshOrder::AllLines => {
                            sb.file_name = self.file_name.clone();
                            sb.mode = self.mode.clone();
                            self.tui.draw_view(&self.view, &sb)
                        }
                    }
                }
            }
        }
    }
}
