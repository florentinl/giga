use std::{
    collections::HashSet,
    fmt::Display,
    io::{Stdout, Write},
};

use termion::{
    clear, color, cursor,
    raw::{IntoRawMode, RawTerminal},
};

use crate::view::View;

use super::{StatusBarInfos, TerminalDrawer};

/// Macro for line number width
const LINE_NUMBER_WIDTH: u16 = 3;


/// # TermionTerminalDrawer is an implementation of the TerminalDrawer trait for the termion crate.
/// The terminal window is split into three parts:
/// - The status bar at the top of the screen
/// - The line numbers on the left of the screen
/// - The actual editor on the rest of the screen
/// To exploit the full potential of the termion crate, the TermionTerminalDrawer acts as a
/// wrapper around the RawTerminal<Stdout> struct provided by termion.
pub struct TermionTerminalDrawer {
    /// The raw terminal output we can write to using termion
    stdout: RawTerminal<Stdout>,
    /// Status bar height
    status_bar_height: u16,
}

impl TerminalDrawer for TermionTerminalDrawer {
    fn terminate(&mut self) {
        // Flush the stdout buffer
        self.stdout.flush().unwrap_or_default();
        // Clear the screen with the "\x1B[3J" escape code (clear screen and scrollback buffer)
        self.print(&clear::All);
        self.print(&"\x1B[3J");
        // Move the cursor to the top left
        self.print(&cursor::Goto(1, 1));
        // Reset the terminal colors
        self.print(&color::Fg(color::Reset));
        self.print(&color::Bg(color::Reset));
        // Disable raw mode
        self.stdout.suspend_raw_mode().unwrap_or_default();
        // Show the terminal cursor
        self.print(&cursor::Show);
    }

    fn clear(&mut self) {
        // Clear the screen
        self.print(&clear::All);
        // Clear the scrollback buffer
        self.print(&"\x1B[3J");
    }

    fn get_term_size(&self) -> (usize, usize) {
        let (x, y) = termion::terminal_size().unwrap_or_default();
        (
            (x - LINE_NUMBER_WIDTH) as usize,
            (y - self.status_bar_height) as usize,
        )
    }

    fn draw(&mut self, view: &View, status_bar_infos: &StatusBarInfos) {
        // Clear the screen
        self.clear();
        // Draw the status bar
        self.draw_status_bar(status_bar_infos);
        // Draw all the lines of the editor
        let all_lines = HashSet::from_iter(0..view.height);
        self.draw_lines(view, all_lines);
        // Move the cursor to the current position
        self.move_cursor(view.cursor);
    }

    fn move_cursor(&mut self, pos: (usize, usize)) {
        let (x, y) = (pos.0 as u16, pos.1 as u16);
        // X is offset by a fixed width for the line numbers plus one space
        let x = x + LINE_NUMBER_WIDTH + 2;
        // Goto is 1-indexed
        self.print(&cursor::Goto(x + 1, y + 1));
    }

    fn draw_lines(&mut self, view: &View, lines: HashSet<usize>) {
        // Draw each line that has changed
        for line in lines {
            // Move the cursor to the beginning of the line
            self.print(&cursor::Goto(1, line as u16 + 1));
            // Clear the line
            self.print(&clear::CurrentLine);
            // Print the line number
            self.draw_line_number(line + view.start_line + 1);
            // Print the line content
            self.print(&view.get_line(line));
        }
        // Move the cursor to its actual position
        self.move_cursor(view.cursor);
    }

    fn draw_status_bar(&mut self, status_bar_infos: &StatusBarInfos) {
        let (width, height) = termion::terminal_size().unwrap_or_default();

        // Move the cursor to the status bar
        self.print(&cursor::Goto(1, height - self.status_bar_height + 1));
        // Set the status bar background color to white
        self.print(&color::Bg(color::White));
        // Set the status bar foreground color to black
        self.print(&color::Fg(color::Black));
        // Print the mode (NORMAL or INSERT)
        self.print(&status_bar_infos.mode);
        // Print the file name at the end of the status bar
        let offset = width as usize - status_bar_infos.file_name.len() - "NORMAL".len();
        self.print(&" ".repeat(offset));
        self.print(&status_bar_infos.file_name);
        // Reset the status bar colors
        self.print(&color::Fg(color::Reset));
        self.print(&color::Bg(color::Reset));
    }
}

impl TermionTerminalDrawer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            stdout: std::io::stdout().into_raw_mode().unwrap(),
            status_bar_height: 1,
        })
    }

    /// # Helper function to print something to the raw
    fn print(&mut self, s: &dyn Display) {
        write!(self.stdout, "{}", s).unwrap_or_default();
        // Flush the stdout buffer
        self.stdout.flush().unwrap_or_default();
    }

    /// # Draw the line numbers
    /// The line numbers are displayed at the left of the screen in blue
    pub fn draw_line_number(&mut self, line: usize) {
        // Set background color to brown-red
        self.print(&color::Bg(color::Rgb(0x88, 0x00, 0x00)));
        // Set foreground color to blue
        self.print(&color::Fg(color::Blue));
        // Print the line number formatted to 3 characters
        self.print(&format!("{:3} ", line));
        // Reset both foreground and background colors
        self.print(&color::Fg(color::Reset));
        self.print(&color::Bg(color::Reset));
        // Leave one space between the line number and the text
        self.print(&cursor::Right(1));
    }
}
