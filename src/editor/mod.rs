//! # Where the magic happens.
//!
//! The editor module contains the main structure of the program: the Editor.
//! The program is architected around an MVC-like pattern. The Editor is the
//! controller, it contains the model (View + Some metadatas) and the view is
//! handled by an implementation of the TerminalDrawer trait.
//!
//! ## Asynchronous architecture
//!
//! There are three threads in the program:
//! - Main thread, responsible for handling user input and modifying the view
//! - Git thread, responsible for computing the diff between the current file and
//!  the current commit and updating the diff field of the editor
//! - The tui thread, responsible for drawing the view to the terminal. This thread listens
//! to both the Main thread and the Git thread (using message passing) and redraws the view when needed.
//!
//! ## The View
//!
//! The view (`View` module) represents the portion of the file that is currently
//! displayed on and it owns the representation of the file (`File` module) in memory.
//! Most of the tasks performed by the editor act on the view (moving the cursor,
//! scrolling, inserting/deleting characters, etc...). The view produces a syntax
//! highlighted representation of the file. The logic for coloring the file is handled
//! in the `Colorizer` struct (in the `color` module) and is performed synchronously
//! by the main thread in the `file` module.
//!
//! ## Command system
//!
//! To act on the editor, the user inputs a `char` to stdin. Which is then
//! parsed into a `Command` enum variant depending on the current mode of the
//! editor. This logic is handled by the `Command` module. The `Command` is
//! then executed by the `Editor` (`Editor::execute` method) which produces
//! a `RefreshOrder` enum variant that represents which portion of the screen
//! needs to be redrawn. The tui thread (`Editor::init_tui_thread` method) then
//! uses this information to redraw the screen.
//!
//! ## Drawing system
//!
//! The drawing system is handled by the `TerminalDrawer` trait. Which exists
//! to abstract the drawing system from the rest of the program (for modularity
//! and mocking purposes). The `TerminalDrawer` trait is implemented in the `terminal`
//! module. The current implementation uses the termion crate to draw to the terminal.
//!
//! ## Git integration
//!
//! If the current file is in a git repository, the editor will display the current
//! branch/commit/tag in the status bar and will display the diff between the current
//! file and the current commit in the left margin. This is done by the `git` module.
//!
//! This task is performed asynchronously by the git thread (`Editor::init_git_thread` method).
//! The git thread computes the diff between the current file and the current commit and stores
//! it in the `diff` field. It then sends a unit signal to the tui thread to notify it that the
//! diff has changed.
//!
//! ## Resizing the terminal
//!
//! To allow the user to resize the terminal, the tui thread listens to the `SIGWINCH` signal
//! and sends a unit signal to the tui thread to notify it that the terminal has been resized.
//! This logic is handled by the `signal` module, which is called in the (`Editor::init_resize_listener` method).
//!
mod command;
mod git;
mod signal;
mod terminal;
mod view;

use std::{
    collections::HashSet,
    fmt::Display,
    io,
    ops::DerefMut,
    path,
    process::exit,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use termion::input::TermRead;

use self::git::Patch;

use {
    command::Command,
    git::{compute_diff, get_ref_name, Diff},
    terminal::{termion::TermionTerminalDrawer, StatusBarInfos, TerminalDrawer},
    view::file::File,
    view::View,
};

/// Macro to create arc mutexes quickly
macro_rules! arc_mutex {
    ($value:expr) => {
        Arc::new(Mutex::new($value))
    };
}

/// Editor structure
/// represents the state of the program
pub struct Editor {
    /// The path of the file being edited
    file_path: String,
    /// The name of the file being edited
    file_name: Arc<Mutex<String>>,
    /// The current view of the file
    view: Arc<Mutex<View>>,
    /// The mode of the editor
    mode: Arc<Mutex<Mode>>,

    /// Git Branch/Commit/Tag if any
    git_ref: Arc<Mutex<Option<String>>>,
    /// Git diff since last commit if any
    pub diff: Arc<Mutex<Option<Diff>>>,
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
    /// Terminate the editor
    Terminate,
    /// No need to refresh the screen
    None,
    /// Refresh the cursor position
    CursorPos,
    /// Refresh the given lines
    Lines(HashSet<usize>),
    /// Refresh the status bar
    StatusBar,
    /// Refresh the git indicators
    GitIndicators,
    /// Refresh the whole screen
    AllLines,
    /// Refresh the screen after a resize event
    Resize,
}

impl Editor {
    /// Create a new editor
    pub fn new(file_path: &str) -> Self {
        let (file_path, file_name, extension) = Self::split_path_name(file_path);
        let ref_name = get_ref_name(&file_path);
        Self {
            file_path,
            file_name: arc_mutex!(file_name),
            view: arc_mutex!(View::new(File::new(&extension), 0, 0)),
            mode: arc_mutex!(Mode::Normal),
            git_ref: arc_mutex!(ref_name),
            diff: Arc::new(Mutex::new(None)),
        }
    }

    /// Open a file in the editor
    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let (file_path, file_name, ext) = Self::split_path_name(path);

        let content = std::fs::read_to_string(path)?;
        let content = File::from_string(&content, &ext);
        let view = Arc::new(Mutex::new(View::new(content, 0, 0)));

        let git_ref = arc_mutex!(get_ref_name(&file_path));

        Ok(Self {
            file_path,
            file_name: arc_mutex!(file_name),
            view,
            mode: arc_mutex!(Mode::Normal),
            git_ref,
            diff: arc_mutex!(None),
        })
    }

    /// Split a file path into a relative path and a name
    fn split_path_name(path: &str) -> (String, String, String) {
        let path = path::Path::new(path);
        let mut file_path = path.parent().unwrap().to_str().unwrap_or_default();
        if file_path.is_empty() {
            file_path = ".";
        }
        let file_name = path.file_name().unwrap().to_str().unwrap_or_default();
        let file_ext = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        (
            String::from(file_path) + "/",
            String::from(file_name),
            String::from(file_ext),
        )
    }

    /// Save the current file
    fn save(&self) {
        let file_name = self.file_name.lock().unwrap();
        if file_name.is_empty() {
            return;
        }
        let path = String::from(&self.file_path) + &file_name;
        let content = self.view.lock().unwrap().dump_file();
        std::fs::write(path.clone() + ".tmp", content).unwrap_or_default();
        std::fs::rename(path.clone() + ".tmp", path).unwrap_or_default();
    }

    /// Rename the current file
    fn rename(&mut self, c: Option<char>) {
        let mut file_name = self.file_name.lock().unwrap();
        match c {
            None => {
                // delete a char
                file_name.pop();
            }
            Some(c) => match c {
                ' ' | '\'' => *file_name = file_name.clone() + "_",
                _ => *file_name = file_name.clone() + &c.to_string(),
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
                // Doesn't matter as self.terminate() never returns
                RefreshOrder::Terminate
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
                self.toggle_rename();
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
            Command::DeleteLine => {
                let mut view = self.view.lock().unwrap();
                view.delete_line();
                RefreshOrder::AllLines
            }
        }
    }

    /// Toggle the mode of the editor between normal and insert
    fn toggle_mode(&mut self) {
        let mut mode = self.mode.lock().unwrap();
        *mode = match mode.clone() {
            Mode::Normal => Mode::Insert,
            Mode::Insert => Mode::Normal,
            Mode::Rename => Mode::Normal,
        }
    }

    fn toggle_rename(&mut self) {
        let mut mode = self.mode.lock().unwrap();
        *mode = match mode.clone() {
            Mode::Normal => Mode::Rename,
            Mode::Rename => Mode::Normal,
            _ => Mode::Normal, // Could not be in insert mode
        }
    }

    /// Initialize git operations
    fn init_git_thread(&mut self, sender: Sender<RefreshOrder>) {
        // Initialize the diff
        self.diff = Arc::new(Mutex::new(None));

        // Spawn a thread to compute the diff in background
        let view = self.view.clone();
        let diff = self.diff.clone();
        let file_path = self.file_path.clone();
        let file_name = self.file_name.clone();
        thread::spawn({
            move || loop {
                let file_name = file_name.lock().unwrap().clone();
                let new_diff =
                    compute_diff(&view.lock().unwrap().dump_file(), &file_path, &file_name).ok();
                let mut current_diff = diff.lock().unwrap();

                // If the diff has changed, redraw the diff markers
                if new_diff != *current_diff {
                    *current_diff = new_diff;
                    sender.send(RefreshOrder::GitIndicators).unwrap();
                }
                // Drop the lock before sleeping
                drop(current_diff);
                thread::sleep(Duration::from_millis(250));
            }
        });
    }

    /// Get the status bar infos
    fn get_status_bar_infos(
        mode: &Arc<Mutex<Mode>>,
        file_name: &Arc<Mutex<String>>,
        git_ref: &Arc<Mutex<Option<String>>>,
    ) -> StatusBarInfos {
        let mode = mode.lock().unwrap();
        let file_name = file_name.lock().unwrap();
        let git_ref = git_ref.lock().unwrap();

        StatusBarInfos {
            file_name: file_name.clone(),
            mode: mode.clone(),
            ref_name: git_ref.clone(),
        }
    }

    /// Refresh the TUI
    fn refresh_tui(
        tui: &mut TermionTerminalDrawer,
        view: &mut View,
        diff: &Arc<Mutex<Option<Vec<Patch>>>>,
        status_bar_infos: &StatusBarInfos,
        refresh_order: RefreshOrder,
    ) {
        match refresh_order {
            RefreshOrder::Terminate => {
                tui.terminate();
                exit(0);
            }
            RefreshOrder::None => (),
            RefreshOrder::CursorPos => tui.move_cursor(view.cursor),
            RefreshOrder::StatusBar => {
                tui.draw_status_bar(status_bar_infos);
                tui.move_cursor(view.cursor)
            }
            RefreshOrder::GitIndicators => {
                let locked_diff = diff.lock().unwrap();
                tui.draw_diff_markers(locked_diff.as_ref().unwrap(), view);
            }
            RefreshOrder::Lines(lines) => tui.draw_lines(view, lines),
            RefreshOrder::AllLines => {
                tui.draw(view, status_bar_infos);
                let locked_diff = diff.lock().unwrap();
                tui.draw_diff_markers(locked_diff.as_ref().unwrap(), view);
            }
            RefreshOrder::Resize => {
                let (width, height) = tui.get_term_size();
                view.resize(height, width);
                Self::refresh_tui(tui, view, diff, status_bar_infos, RefreshOrder::AllLines);
            }
        }
    }

    /// # Initialize the tui drawing thread
    /// This thread listens from three channels:
    /// - `cmd_rx`: commands refresh orders from the main thread
    /// - `diff_changed`: a channel that is notified when the diff has changed
    /// - `resize_rx`: a channel that is notified when the terminal has been resized
    /// It then draws the TUI accordingly.
    fn init_tui_thread(&mut self, refresh_receiver: Receiver<RefreshOrder>) {
        let mut tui = TermionTerminalDrawer::new();

        // Get the terminal size and initialize the view
        let (width, height) = tui.get_term_size();
        let mut locked_view = self.view.lock().unwrap();
        locked_view.resize(height, width);

        // Get the initial status bar infos
        let status_bar_infos =
            Self::get_status_bar_infos(&self.mode, &self.file_name, &self.git_ref);

        // Draw the initial TUI
        tui.draw(&locked_view, &status_bar_infos);

        // Spawn a thread to draw the TUI in background
        let view = self.view.clone();
        let diff = self.diff.clone();
        let mode = self.mode.clone();
        let file_name = self.file_name.clone();
        let git_ref = self.git_ref.clone();
        thread::spawn({
            move || {
                loop {
                    if let Ok(refresh_order) = refresh_receiver.recv() {
                        let mut locked_view = view.lock().unwrap();
                        let status_bar_infos =
                            Self::get_status_bar_infos(&mode, &file_name, &git_ref);

                        Self::refresh_tui(
                            &mut tui,
                            locked_view.deref_mut(),
                            &diff,
                            &status_bar_infos,
                            refresh_order,
                        );
                    }

                    // Sleep for 1000/120 ms to avoid hogging the CPU
                    thread::sleep(Duration::from_millis(1000 / 120));
                }
            }
        });
    }

    /// Run the editor loop
    pub fn run(&mut self) {
        let (refresh_sender, refresh_receiver) = mpsc::channel::<RefreshOrder>();

        // Initialize git operations if needed
        let git_ref = self.git_ref.lock().unwrap().clone();
        if git_ref.is_some() {
            self.init_git_thread(refresh_sender.clone())
        }

        // Initialize the resize signal handler
        signal::init_resize_listener(refresh_sender.clone());

        // Initialize the TUI thread
        self.init_tui_thread(refresh_receiver);

        // Main loop of the editor (waiting for key events)
        for key in io::stdin().keys().flatten() {
            let mode = self.mode.lock().unwrap().clone();
            // Parse the key
            if let Ok(cmd) = Command::parse(key, &mode) {
                // Execute the command
                let refresh_order = self.execute(cmd);

                // Send the refresh order to the TUI
                refresh_sender.send(refresh_order).unwrap();
            }
        }
    }
}
