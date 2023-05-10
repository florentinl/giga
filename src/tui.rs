extern crate termion;
use crate::file;
use crate::view::View;

use termion::clear;
use termion::cursor;
use termion::color;

pub fn get_term_size() -> (u16, u16) {
    termion::terminal_size().unwrap()
}

/// initialize the terminal
pub fn init() {
    // clear the screen
    print!("{}{}", cursor::Goto(1, 1), clear::All);
}

fn status_bar(file_name: String, height: u16, width: u16) -> () {
    let padding = width - file_name.len() as u16;
    print!("{}{}{}{}{}{}{}", 
    cursor::Goto(1, height+1),
    color::Bg(color::White),
    color::Fg(color::Black),
    file_name + &" ".repeat(padding as usize),
    cursor::Goto(width, height+1),
    color::Fg(color::Reset),
    color::Bg(color::Reset),

);
}

pub fn display(view: &View, file_name: &Option<String>) {
    let height = view.height;
    let width = view.width;
    for line in 0..height {
        print!(
            "{}{}",
            cursor::Goto(1, (line + 1) as u16),
            view.get_line(line)
        );
    }
    // print the status bar
    let name = match file_name {
        Some(name) => name.clone(),
        None => String::from("New file"),
    };
    status_bar(name, height as u16, width as u16);
    print!("{}", cursor::Goto(1, 1));
}
