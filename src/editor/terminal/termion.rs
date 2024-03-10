use std::{
    collections::{HashMap, HashSet},
    io::{Stdout, Write},
};

use termion::{
    clear, color, cursor,
    raw::{IntoRawMode, RawTerminal},
};

use crate::editor::view::{file::git::PatchType, FileView, View};

use super::{StatusBarInfos, TerminalDrawer};

/// Macro for line number width
const LINE_NUMBER_WIDTH: u16 = 3;
const STATUS_BAR_HEIGHT: u16 = 1;

/// Macro for writing to the terminal
macro_rules! print {
    ($stdout:expr, $($arg:tt)*) => {
        write!($stdout, "{}", $($arg)*).unwrap_or_default();
    };
}

/// # TermionTerminalDrawer is an implementation of the TerminalDrawer trait for the termion crate.
/// The terminal window is split into three parts:
/// - The status bar at the top of the screen
/// - The line numbers on the left of the screen
/// - The actual editor on the rest of the screen
/// To exploit the full potential of the termion crate, the TermionTerminalDrawer acts as a
/// wrapper around the `RawTerminal<Stdout>` struct provided by termion.
pub struct TermionTerminalDrawer {
    /// The raw terminal output we can write to using termion
    stdout: RawTerminal<Stdout>,
}

impl TerminalDrawer for TermionTerminalDrawer {
    fn terminate(&mut self) {
        // Clear the screen with the "\x1B[3J" escape code (clear screen and scrollback buffer)
        print!(self.stdout, clear::All);
        print!(self.stdout, "\x1B[3J");
        // Move the cursor to the top left
        print!(self.stdout, cursor::Goto(1, 1));
        // Reset the terminal colors
        print!(self.stdout, color::Fg(color::Reset));
        print!(self.stdout, color::Bg(color::Reset));
        // Disable raw mode
        self.stdout.suspend_raw_mode().unwrap_or_default();
        // Show the terminal cursor
        print!(self.stdout, cursor::Show);

        // Flush the stdout buffer
        self.stdout.flush().unwrap_or_default();
    }

    fn clear(&mut self) {
        // Clear the screen
        print!(self.stdout, clear::All);
        // Clear the scrollback buffer
        print!(self.stdout, "\x1B[3J");
    }

    fn get_term_size(&self) -> (usize, usize) {
        let (x, y) = termion::terminal_size().unwrap_or_default();
        (
            (x - LINE_NUMBER_WIDTH - 2) as usize,
            (y - STATUS_BAR_HEIGHT) as usize,
        )
    }

    fn draw(&mut self, view: &View, status_bar_infos: &StatusBarInfos) {
        // Hide the terminal cursor
        print!(self.stdout, cursor::Hide);
        // Draw the status bar
        self.draw_status_bar(status_bar_infos);
        // Draw all the lines of the editor
        let all_lines = HashSet::from_iter(0..view.height);
        self.draw_lines(view, all_lines);
        // Show the cursor
        print!(self.stdout, cursor::Show);
        // Move the cursor to the current position
        self.move_cursor(view.cursor);
    }

    fn move_cursor(&mut self, pos: (usize, usize)) {
        let (x, y) = (pos.0 as u16, pos.1 as u16);
        // X is offset by a fixed width for the line numbers plus one space
        let x = x + LINE_NUMBER_WIDTH + 2;
        // Goto is 1-indexed
        print!(self.stdout, cursor::Goto(x + 1, y + 1));

        self.flush();
    }

    fn draw_lines(&mut self, view: &View, lines: HashSet<usize>) {
        // Draw each line that has changed
        for line in lines {
            // Move the cursor to the beginning of the line
            print!(self.stdout, cursor::Goto(1, line as u16 + 1));
            // Print the line number
            self.draw_line_number(line + view.start_line + 1);
            // Leave one space for git diff markers
            print!(self.stdout, cursor::Right(1));
            // Print the line content
            print!(self.stdout, view.get_line(line));
            // Clear the rest of the line
            print!(self.stdout, clear::UntilNewline);
        }
        // Move the cursor to its actual position
        self.move_cursor(view.cursor);
    }

    // The status bar is at the bottom of the screen and displays the following information:
    // - The current mode (NORMAL/INSERT/RENAME) (left)
    // - The current file name (in the middle)
    // - The current git branch (if we are in a git) (right)
    fn draw_status_bar(&mut self, status_bar_infos: &StatusBarInfos) {
        let (width, height) = termion::terminal_size().unwrap_or_default();

        // Move the cursor to the status bar
        print!(self.stdout, cursor::Goto(1, height - STATUS_BAR_HEIGHT + 1));
        // Set the status bar background color to white
        print!(self.stdout, color::Bg(color::White));
        // Set the status bar foreground color to black
        print!(self.stdout, color::Fg(color::Black));
        // Print the mode (NORMAL or INSERT)
        print!(self.stdout, " ");
        print!(self.stdout, status_bar_infos.mode);
        // Print the file name in the middle of the status bar
        let offset = (width as usize - status_bar_infos.file_name.len()) / 2 - " NORMAL".len();
        print!(self.stdout, " ".repeat(offset));
        print!(self.stdout, status_bar_infos.file_name);
        // Print the git branch if we are in a git repository at the right of the status bar
        if let Some(git_branch) = &status_bar_infos.ref_name {
            let offset = width as usize
                - "NORMAL".len() // All modes have the same length
                - status_bar_infos.file_name.len()
                - offset
                - 2
                - git_branch.len();
            print!(self.stdout, " ".repeat(offset));
            print!(self.stdout, git_branch);
        } else {
            // If we are not in a git repository, we still need to print spaces to fill the status bar
            let offset = width as usize
                - "NORMAL".len() // All modes have the same length
                - status_bar_infos.file_name.len()
                - 2
                - offset;
            print!(self.stdout, " ".repeat(offset));
        }
        print!(self.stdout, " ");
        // Reset the status bar colors
        print!(self.stdout, color::Fg(color::Reset));
        print!(self.stdout, color::Bg(color::Reset));

        self.flush();
    }

    /// Draw the diff markers on the left of the screen
    /// - '▐' (green) for added lines
    /// - '▗' (red) for removed lines
    /// - '▐' (yellow) for modified lines
    /// - ' ' (default) for unchanged lines
    fn draw_diff_markers(&mut self, diff: HashMap<usize, PatchType>, view: &View) {
        for line in 0..view.height {
            // Go to the beginning of the line
            print!(
                self.stdout,
                cursor::Goto(LINE_NUMBER_WIDTH + 1, line as u16 + 1)
            );
            // Print the diff marker
            match diff.get(&(line + view.start_line)) {
                Some(PatchType::Added) => {
                    print!(self.stdout, color::Fg(color::Green));
                    print!(self.stdout, "▐");
                }
                Some(PatchType::Deleted) => {
                    print!(self.stdout, color::Fg(color::Red));
                    print!(self.stdout, "▗");
                }
                Some(PatchType::Changed) => {
                    print!(self.stdout, color::Fg(color::Yellow));
                    print!(self.stdout, "▐");
                }
                _ => {
                    print!(self.stdout, " ");
                }
            }
        }
        self.move_cursor(view.cursor);
    }
}

impl TermionTerminalDrawer {
    pub fn new() -> Box<Self> {
        let mut drawer = Self {
            stdout: std::io::stdout().into_raw_mode().unwrap(),
        };
        drawer.clear();
        Box::new(drawer)
    }

    /// # Helper funtion to flush the stdout buffer
    fn flush(&mut self) {
        self.stdout.flush().unwrap_or_default();
    }

    /// # Draw the line numbers
    /// The line numbers are displayed at the left of the screen in blue
    fn draw_line_number(&mut self, line: usize) {
        // Set foreground color to blue
        print!(self.stdout, color::Fg(color::Blue));
        // Print the line number formatted to 3 characters
        print!(self.stdout, format!("{:3} ", line));
        // Reset both foreground and background colors
        print!(self.stdout, color::Fg(color::Reset));
        print!(self.stdout, color::Bg(color::Reset));
    }
}
