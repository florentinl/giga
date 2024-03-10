//! # Provides the terminal drawing logic
//!
//! The terminal drawing logic is abstracted the `TerminalDrawer` trait and
//! implemented for the termion crate using the `TermionTerminalDrawer` struct
//! in the `termion` module.

pub mod termion;

use crate::editor::{view::View, Mode};
use std::collections::{HashMap, HashSet};

use super::view::file::git::PatchType;

/// A TerminalDrawer instance is responsible for drawing the editor on the terminal
pub trait TerminalDrawer {
    /// Terminate the TerminalDrawer instance (potentially cleanup the terminal)
    fn terminate(&mut self);
    /// Get the terminal dimensions
    fn get_term_size(&self) -> (usize, usize);
    /// Clear the terminal
    fn clear(&mut self);
    /// (Re)Draw the whole editor
    fn draw(&mut self, view: &View, status_bar_infos: &StatusBarInfos);
    /// Move the cursor to the given position
    fn move_cursor(&mut self, pos: (usize, usize));
    /// (Re)Draw only the lines that have changed
    fn draw_lines(&mut self, view: &View, lines: HashSet<usize>);
    /// (Re)Draw the status bar
    fn draw_status_bar(&mut self, status_bar_infos: &StatusBarInfos);
    /// (Re)Draw the diff markers on the left of the editor
    fn draw_diff_markers(&mut self, diff: HashMap<usize, PatchType>, view: &View);
}

/// Information that go in the status bar
pub struct StatusBarInfos {
    pub file_name: String,
    pub mode: Mode,
    pub ref_name: Option<String>,
}
