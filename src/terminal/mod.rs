use std::collections::HashSet;

use crate::{editor::Mode, view::View};

pub mod termion;
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
}

/// Information that go in the status bar
pub struct StatusBarInfos {
    pub file_path: String,
    pub file_name: String,
    pub mode: Mode,
}
