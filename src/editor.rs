use std::{
    collections::HashSet,
    fmt::Display,
    process::exit,
    sync::{Arc, Mutex},
};

use crate::{
    command::Command,
    file::File,
    git::{compute_diff, get_ref_name, Diff},
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
    view: Arc<Mutex<View>>,
    /// The Tui responsible for drawing the editor
    tui: Box<dyn TerminalDrawer>,
    /// The mode of the editor
    mode: Mode,

    /// Git Branch/Commit/Tag if any
    git_ref: Option<String>,
    /// Git diff since last commit if any
    pub diff: Arc<Mutex<Option<Diff>>>,
    /// Git thread handle
    git_thread: Option<std::thread::JoinHandle<()>>,
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
    pub fn new(file_path: &str) -> Self {
        let (file_path, file_name) = Self::split_path_name(file_path);
        let ref_name = get_ref_name(&file_path);
        Self {
            file_path,
            file_name,
            view: Arc::new(Mutex::new(View::new(File::new(), 0, 0))),
            tui: TermionTerminalDrawer::new(),
            mode: Mode::Normal,
            git_ref: ref_name,
            diff: Arc::new(Mutex::new(None)),
            git_thread: None,
        }
    }

    /// Open a file in the editor
    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let content = File::from_string(&content);
        let view = Arc::new(Mutex::new(View::new(content, 0, 0)));

        let (path, file_name) = Self::split_path_name(path);

        let git_ref = get_ref_name(&path);

        Ok(Self {
            file_path: path.to_string(),
            file_name: file_name.to_string(),
            view,
            tui: TermionTerminalDrawer::new(),
            mode: Mode::Normal,
            git_ref: git_ref,
            diff: Arc::new(Mutex::new(None)),
            git_thread: None,
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
        let content = self.view.lock().unwrap().dump_file();
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
            ref_name: self.git_ref.clone(),
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
                let scroll = self.view.lock().unwrap().navigate(x, y);
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
                let mut view = self.view.lock().unwrap();
                let scroll = view.insert(c);
                if scroll {
                    RefreshOrder::AllLines
                } else {
                    // Refresh only the current line: self.view.cursor.1
                    RefreshOrder::Lines(HashSet::from_iter(vec![view.cursor.1]))
                }
            }
            Command::InsertNewLine => {
                let mut view = self.view.lock().unwrap();
                let scroll = view.insert_new_line();
                if scroll {
                    RefreshOrder::AllLines
                } else {
                    // Refresh only the lines below the cursor
                    RefreshOrder::Lines(HashSet::from_iter(view.cursor.1 - 1..view.height))
                }
            }
            Command::Delete => {
                let mut view = self.view.lock().unwrap();
                let scroll = view.delete();
                if scroll {
                    // If we scroll (because we deleted a char at the left of the view),
                    // we need to refresh all lines.
                    RefreshOrder::AllLines
                } else {
                    // Refresh only the lines below the cursor
                    RefreshOrder::Lines(HashSet::from_iter(view.cursor.1..view.height))
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

    ///

    /// Run the editor loop
    pub fn run(&mut self) {
        // set view size
        let (width, height) = self.tui.get_term_size();

        self.view.lock().unwrap().resize(height, width);
        // draw initial view
        self.tui.clear();
        self.tui.draw(&self.view.lock().unwrap(), &self.get_status_bar_infos());

        // If we are in a git repo (i.e. if self.git_ref is not None),
        // compute the diff a first time
        if let Some(_) = self.git_ref {
            let diff = compute_diff(
                &self.view.lock().unwrap().dump_file(),
                &self.file_path,
                &self.file_name,
            )
            .unwrap();
            self.diff = Arc::new(Mutex::new(Some(diff)));
            let view = self.view.clone();
            let diff = self.diff.clone();
            let file_path = self.file_path.clone();
            let file_name = self.file_name.clone();
            // Spawn a thread to compute the diff in background
            let diff_thread = std::thread::spawn({
                move || loop {
                    let new_diff =
                        compute_diff(&view.lock().unwrap().dump_file(), &file_path, &file_name);
                    *diff.lock().unwrap() = new_diff.ok();
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            });
            self.git_thread = Some(diff_thread);
        }

        let stdin = std::io::stdin().keys();

        for c in stdin {
            if let Ok(c) = c {
                if let Ok(cmd) = Command::parse(c, &self.mode) {
                    let refresh_order = self.execute(cmd);
                    let view = self.view.lock().unwrap();
                    match refresh_order {
                        RefreshOrder::None => (),
                        RefreshOrder::CursorPos => self.tui.move_cursor(view.cursor),
                        RefreshOrder::StatusBar => {
                            self.tui.draw_status_bar(&self.get_status_bar_infos());
                            self.tui.move_cursor(view.cursor)
                        }
                        RefreshOrder::Lines(lines) => self.tui.draw_lines(&view, lines),
                        RefreshOrder::AllLines => {
                            self.tui.draw(&view, &self.get_status_bar_infos())
                        }
                    }
                    drop(view);
                }
            }
        }
    }
}
