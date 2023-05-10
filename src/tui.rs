extern crate termion;
use crate::view::View;

use termion::clear;
use termion::cursor;

pub fn get_term_size() -> (u16, u16) {
    termion::terminal_size().unwrap()
}

/// initialize the terminal
fn init() {
    // clear the screen
    print!("{}{}", cursor::Goto(1, 1), clear::All);
}

pub fn display(view: &View) {
    init();
    let height = view.height;
    for line in 0..height {
        print!(
            "{}{}",
            cursor::Goto(1, (line + 1) as u16),
            view.get_line(line)
        );
    }
    print!("{}", cursor::Goto(1, 1));
}
