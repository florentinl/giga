use std::{
    collections::HashSet,
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
const STATUS_BAR_HEIGHT: u16 = 1;

/// Define Macro for printing to the terminal
macro_rules! print_to_term {
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
/// wrapper around the RawTerminal<Stdout> struct provided by termion.
pub struct TermionTerminalDrawer {
    /// The raw terminal output we can write to using termion
    stdout: RawTerminal<Stdout>,
}

impl TerminalDrawer for TermionTerminalDrawer {
    fn terminate(&mut self) {
        // Clear the screen with the "\x1B[3J" escape code (clear screen and scrollback buffer)
        print_to_term!(self.stdout, clear::All);
        print_to_term!(self.stdout, "\x1B[3J");
        // Move the cursor to the top left
        print_to_term!(self.stdout, cursor::Goto(1, 1));
        // Reset the terminal colors
        print_to_term!(self.stdout, color::Fg(color::Reset));
        print_to_term!(self.stdout, color::Bg(color::Reset));
        // Disable raw mode
        self.stdout.suspend_raw_mode().unwrap_or_default();
        // Show the terminal cursor
        print_to_term!(self.stdout, cursor::Show);

        // Flush the stdout buffer
        self.stdout.flush().unwrap_or_default();
    }

    fn clear(&mut self) {
        // Clear the screen
        print_to_term!(self.stdout, clear::All);
        // Clear the scrollback buffer
        print_to_term!(self.stdout, "\x1B[3J");
    }

    fn get_term_size(&self) -> (usize, usize) {
        let (x, y) = termion::terminal_size().unwrap_or_default();
        (
            (x - LINE_NUMBER_WIDTH - 2) as usize,
            (y - STATUS_BAR_HEIGHT) as usize,
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
        print_to_term!(self.stdout, cursor::Goto(x + 1, y + 1));

        self.flush();
    }

    fn draw_lines(&mut self, view: &View, lines: HashSet<usize>) {
        // Draw each line that has changed
        for line in lines {
            // Move the cursor to the beginning of the line
            print_to_term!(self.stdout, cursor::Goto(1, line as u16 + 1));
            // Clear the line
            print_to_term!(self.stdout, clear::CurrentLine);
            // Print the line number
            self.draw_line_number(line + view.start_line + 1);
            // Print the line content
            print_to_term!(self.stdout, view.get_line(line));
        }
        // Move the cursor to its actual position
        self.move_cursor(view.cursor);
    }

    fn draw_status_bar(&mut self, status_bar_infos: &StatusBarInfos) {
        let (width, height) = termion::terminal_size().unwrap_or_default();

        // Move the cursor to the status bar
        print_to_term!(self.stdout, cursor::Goto(1, height - STATUS_BAR_HEIGHT + 1));
        // Set the status bar background color to white
        print_to_term!(self.stdout, color::Bg(color::White));
        // Set the status bar foreground color to black
        print_to_term!(self.stdout, color::Fg(color::Black));
        // Print the mode (NORMAL or INSERT)
        print_to_term!(self.stdout, status_bar_infos.mode);
        // Print the file name at the end of the status bar
        let offset = width as usize - status_bar_infos.file_name.len() - "NORMAL".len();
        print_to_term!(self.stdout, " ".repeat(offset));
        print_to_term!(self.stdout, status_bar_infos.file_name);
        // Reset the status bar colors
        print_to_term!(self.stdout, color::Fg(color::Reset));
        print_to_term!(self.stdout, color::Bg(color::Reset));

        self.flush();
    }
}

impl TermionTerminalDrawer {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            stdout: std::io::stdout().into_raw_mode().unwrap(),
        })
    }

    /// # Helper funtion to flush the stdout buffer
    fn flush(&mut self) {
        self.stdout.flush().unwrap_or_default();
    }

    /// # Draw the line numbers
    /// The line numbers are displayed at the left of the screen in blue
    pub fn draw_line_number(&mut self, line: usize) {
        // Set foreground color to blue
        print_to_term!(self.stdout, color::Fg(color::Blue));
        // Print the line number formatted to 3 characters
        print_to_term!(self.stdout, format!("{:3} ", line));
        // Reset both foreground and background colors
        print_to_term!(self.stdout, color::Fg(color::Reset));
        print_to_term!(self.stdout, color::Bg(color::Reset));
        // Leave one space between the line number and the text
        print_to_term!(self.stdout, cursor::Right(1));
    }
}