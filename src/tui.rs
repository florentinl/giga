extern crate termion;
use crate::editor::Mode;
use crate::view::View;
use std::io::Write;
use termion::clear;
use termion::color;
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

const LINE_NUMBER_WIDTH: u16 = 4;

/// Terminal User Interface
/// Responsible for drawing the editor and handling user input using termion in raw mode
pub struct Tui {
    // Async input reader (stdin)
    pub stdout: RawTerminal<std::io::Stdout>,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            stdout: std::io::stdout().into_raw_mode().unwrap(),
        }
    }

    /// Get the terminal size
    /// Returns a tuple of (width, height)
    pub fn get_term_size(&self) -> (u16, u16) {
        termion::terminal_size().unwrap_or_default()
    }

    /// Clear the screen
    pub fn clear(&mut self) {
        // Clear the screen with the "\x1B[3J" escape code (clear screen and scrollback buffer)
        write!(self.stdout, "{}", clear::All).unwrap_or_default();
        write!(self.stdout, "{}", "\x1B[3J").unwrap_or_default();
        // Move the cursor to the top left
        write!(self.stdout, "{}", cursor::Goto(1, 1)).unwrap_or_default();
    }

    /// Cleanup the terminal before exiting the program
    /// This will :
    /// - Flush the stdout buffer
    /// - Clear the screen
    /// - Move the cursor to the top left
    /// - Reset the terminal colors
    /// - Reset the terminal cursor
    /// - Disable raw mode
    /// - Show the cursor
    pub fn cleanup(&mut self) {
        // Flush the stdout buffer
        self.stdout.flush().unwrap_or_default();
        // Clear the screen with the "\x1B[3J" escape code (clear screen and scrollback buffer)
        write!(self.stdout, "{}", clear::All).unwrap_or_default();
        write!(self.stdout, "{}", "\x1B[3J").unwrap_or_default();
        // Move the cursor to the top left
        write!(self.stdout, "{}", cursor::Goto(1, 1)).unwrap_or_default();
        // Reset the terminal colors
        write!(self.stdout, "{}", color::Fg(color::Reset)).unwrap_or_default();
        write!(self.stdout, "{}", color::Bg(color::Reset)).unwrap_or_default();
        // Reset the terminal cursor
        write!(self.stdout, "{}", cursor::Show).unwrap_or_default();
        self.stdout.suspend_raw_mode().unwrap_or_default();
    }

    /// Draw the status bar
    /// The status bar is displayed at the bottom of the screen
    /// It contains the current mode and the file name
    pub fn draw_status_bar(
        &mut self,
        file_name: String,
        mode: &Mode,
        height: u16,
        width: u16,
    ) {
        let mode: String = match mode {
            Mode::Normal => "NORMAL ".to_string(),
            Mode::Insert => "INSERT ".to_string(),
        };
        let padding = width - file_name.len() as u16 - mode.len() as u16;
        write!(
            self.stdout,
            "{}{}{}{}{}{}{}{}{}",
            cursor::Goto(1, height + 1),
            color::Bg(color::White),
            color::Fg(color::Black),
            mode,
            file_name,
            " ".repeat(padding as usize),
            cursor::Goto(width, height + 1),
            color::Fg(color::Reset),
            color::Bg(color::Reset),
        )
        .unwrap_or_default();
    }

    /// Draw the line numbers
    /// The line numbers are displayed at the left of the screen in blue
    pub fn draw_line_numbers(&mut self, line: usize) {
        let number = format!("{:3} ", line);

        write!(
            self.stdout,
            "{}{}{}{}{}",
            cursor::Goto(1, (line + 1) as u16),
            color::Fg(color::Blue),
            number,
            cursor::Goto(LINE_NUMBER_WIDTH, (line + 1) as u16),
            color::Fg(color::Reset),
        )
        .unwrap_or_default();
    }

    /// Draw the view on the screen
    /// The view is the portion of the file being displayed
    pub fn draw_view(&mut self, view: &View, file_name: &Option<String>, mode: &Mode) {
        self.clear();
        let height = view.height;
        let width = view.width;
        for line in 0..height {
            self.draw_line_numbers(line);
            write!(
                self.stdout,
                "{}{}",
                cursor::Goto(LINE_NUMBER_WIDTH + 1, (line + 1) as u16),
                view.get_line(line)
            )
            .unwrap_or_default();
        }
        // print the status bar
        let name = file_name.clone().unwrap_or("New File".to_string());
        self.draw_status_bar(name, mode, height as u16, width as u16);
        print!("{}", cursor::Goto(1, 1));

        // move the cursor to the correct position
        let (x, y) = view.cursor;
        print!(
            "{}",
            cursor::Goto(x as u16 + LINE_NUMBER_WIDTH as u16 + 1, y as u16 + 1)
        );
        std::io::stdout().flush().unwrap_or_default();
    }
}