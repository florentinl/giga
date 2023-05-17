use std::{collections::HashSet, process::exit};

use crate::{
    command::Command,
    file::{File},
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
}

pub enum RefreshOrder {
    CursorPos,
    Lines(HashSet<u16>),
    StatusBar,
    AllLines,
}

impl Editor {
    /// Create a new editor
    pub fn new(file_name: Option<&str>) -> Self {
        Self {
            path: file_name.unwrap_or_default().to_string(),
            file_name: file_name.unwrap_or_default().to_string(),
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

        Ok(Self {
            path: path.to_string(),
            file_name: path.to_string(),
            view,
            tui: Tui::new(),
            mode: Mode::Normal,
        })
    }
    fn save(&self) {
        let path = &self.file_name; {
            let content = self.view.dump_file();
            std::fs::write(path.clone() + ".tmp", content).unwrap_or_default();
            std::fs::rename(path.clone() + ".tmp", path).unwrap_or_default();
        }
    }

    fn terminate(&mut self) {
        self.tui.cleanup();
        exit(0);
    }

    /// Execute an editor command
    /// - Quit: exit the program
    /// - Move: move the cursor
    /// - Save: save the file
    /// - ToggleMode: toogle editor mode
    /// - Insert: insert a character
    /// - Delete: delete a character
    fn execute(&mut self, cmd: Command) -> RefreshOrder {
        match cmd {
            Command::Quit => {
                self.terminate();
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
            Command::ToggleMode => {
                self.toggle_mode();
                RefreshOrder::StatusBar
            }
            Command::Insert(c) => {
                let scroll = self.view.insert(c);
                if scroll {
                    return RefreshOrder::AllLines;
                }
                let y = self.view.cursor.1;
                let mut lines_to_refresh = HashSet::new();
                lines_to_refresh.insert(y as u16);
                RefreshOrder::Lines(lines_to_refresh)
            }
            Command::InsertNewLine => {
                let y = self.view.cursor.1;
                let mut lines_to_refresh = HashSet::new();
                let scroll = self.view.insert_new_line();
                if scroll {
                    return RefreshOrder::AllLines;
                } else {
                    for i in y..self.view.height {
                        lines_to_refresh.insert(i as u16);
                    }
                }
                RefreshOrder::Lines(lines_to_refresh)
            }
            Command::Delete => {
                let scroll = self.view.delete();
                if scroll {
                    return RefreshOrder::AllLines;
                }
                let y = self.view.cursor.1;
                let mut lines_to_refresh = HashSet::new();
                for i in y..self.view.height {
                    lines_to_refresh.insert(i as u16);
                }
                RefreshOrder::Lines(lines_to_refresh)
            }
            Command::CommandBlock(cmds) => {
                let mut refr: RefreshOrder = RefreshOrder::StatusBar;
                let mut lines_to_refresh: HashSet<u16> = HashSet::new();
                cmds.into_iter().for_each(|cmd| {
                    refr = self.execute(cmd);
                    match &refr {
                        RefreshOrder::Lines(lines) => {
                            lines_to_refresh.extend(lines);
                        }
                        RefreshOrder::CursorPos | RefreshOrder::StatusBar => {
                            refr = RefreshOrder::AllLines
                        } // on command, we can refresh all lines as it may be undo / redo
                        _ => {}
                    }
                });
                refr
            }
        }
    }

    /// Toggle the mode of the editor between normal and insert
    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Normal => Mode::Insert,
            Mode::Insert => Mode::Normal,
        }
    }

    /// Run the editor loop
    pub fn run(&mut self) {
        let mut sb = StatusBar {
            path: ".".to_string(),
            file_name: self.file_name.clone(),
            mode: self.mode.clone(),
        };
        // set view size
        let (width, height) = self.tui.get_term_size();
        // height - 1 to leave space for the status bar
        // width - 3 to leave space for the line numbers
        self.view
            .resize((height - 1) as usize, (width - 4) as usize);
        // draw initial view
        self.tui.clear();
        self.tui.draw_view(&self.view, &sb);

        let stdin = std::io::stdin().keys();

        for c in stdin {
            if let Ok(c) = c {
                if let Ok(cmd) = Command::parse(c, &self.mode) {
                    let refresh_order = self.execute(cmd);
                    match refresh_order {
                        RefreshOrder::AllLines => {
                            self.tui.draw_view(&self.view, &sb)
                        }
                        RefreshOrder::StatusBar => {
                            sb.file_name = self.file_name.clone();
                            sb.mode = self.mode.clone();
                            self.tui.draw_status_bar(&sb, height, width)
                        }
                        RefreshOrder::CursorPos => {
                            let (x, y) = self.view.cursor;
                            self.tui.move_cursor(x, y)
                        }
                        RefreshOrder::Lines(lines) => self.tui.refresh_lines(&self.view, lines),
                    }
                }
            }
        }
    }
}
