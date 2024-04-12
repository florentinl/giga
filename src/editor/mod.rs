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
//! module. The current implementation uses the tui crate to draw to the terminal.
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
mod view;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

use std::io;

pub mod tui;

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use self::view::FileView;

use view::View;

/// Macro to create arc mutexes quickly
macro_rules! arc_mutex {
    ($value:expr) => {
        Arc::new(Mutex::new($value))
    };
}

/// Editor structure
/// represents the state of the program
pub struct Editor {
    /// The current view of the file
    view: Arc<Mutex<View>>,
    /// The mode of the editor
    mode: Arc<Mutex<Mode>>,

    exit: bool,
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

impl Editor {
    /// Open a file in the editor
    pub fn open(path: &str) -> Self {
        Self {
            view: arc_mutex!(View::new(path)),
            mode: arc_mutex!(Mode::Normal),
            exit: false,
        }
    }

    /// Save the current file
    fn save(&self) {
        let view = self.view.lock().unwrap();
        let file_path = view.file_path();
        let content = view.dump_file();

        std::fs::write(file_path.clone() + ".tmp", content).unwrap_or_default();
        std::fs::rename(file_path.clone() + ".tmp", file_path).unwrap_or_default();
    }

    /// Rename the current file
    fn rename(&mut self, c: Option<char>) {
        let mut view = self.view.lock().unwrap();
        let mut file_name = view.file_name();
        match c {
            None => {
                file_name.pop();
                view.set_file_name(file_name);
            }
            Some(c) => {
                file_name.push(c);
                view.set_file_name(file_name);
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    /// Run the editor loop
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let mode = self.mode.lock().unwrap().clone();
        match mode {
            Mode::Normal => self.handle_normal_key_events(key_event),
            Mode::Insert => self.handle_insert_key_events(key_event),
            Mode::Rename => self.handle_rename_key_events(key_event),
        }
    }

    fn handle_normal_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('i') => {
                *self.mode.lock().unwrap() = Mode::Insert;
            }
            KeyCode::Char('r') => {
                *self.mode.lock().unwrap() = Mode::Rename;
            }
            KeyCode::Char('s') => {
                self.save();
            }
            KeyCode::Char('q') => {
                self.exit();
            }
            _ => {}
        }
    }
    fn handle_insert_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                *self.mode.lock().unwrap() = Mode::Normal;
            }
            _ => {}
        }
    }
    fn handle_rename_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                self.rename(Some(c));
            }
            KeyCode::Backspace => {
                self.rename(None);
            }
            KeyCode::Enter => {
                *self.mode.lock().unwrap() = Mode::Normal;
            }
            _ => {}
        }
    }
}

impl Widget for &Editor {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(self.view.lock().unwrap().get_file_name_padded());
        let mode = self.mode.lock().unwrap().clone();
        let style = match mode {
            Mode::Normal => Style::default().fg(Color::Green),
            Mode::Insert => Style::default().fg(Color::Yellow),
            Mode::Rename => Style::default().fg(Color::Red),
        };
        let mode_text = Title::from(Line::from(vec![
            " ".into(),
            Span::styled(self.mode.lock().unwrap().to_string(), style),
            " ".into(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                mode_text
                    .alignment(Alignment::Left)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::ROUNDED);

        let file_text = Text::from(self.view.lock().unwrap().dump_file());

        Paragraph::new(file_text.white())
            .block(block)
            .render(area, buf);
    }
}
