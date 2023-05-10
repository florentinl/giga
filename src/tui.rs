extern crate termion;
use crate::view::View;

use termion::clear;
use termion::color;
use termion::cursor;

const line_number_width: u16 = 4;

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
    print!(
        "{}{}{}{}{}{}{}",
        cursor::Goto(1, height + 1),
        color::Bg(color::White),
        color::Fg(color::Black),
        file_name + &" ".repeat(padding as usize),
        cursor::Goto(width, height + 1),
        color::Fg(color::Reset),
        color::Bg(color::Reset),
    );
}

fn line_number(line: usize) -> () {
    let number = match line {
        0..=9 => format!("  {}", line),
        10..=99 => format!(" {}", line),
        _ => line.to_string(),
    };

    print!(
        "{}{}{}{}{}",
        cursor::Goto(1, (line + 1) as u16),
        color::Fg(color::Blue),
        number,
        cursor::Goto(line_number_width, (line + 1) as u16),
        color::Fg(color::Reset),
    );
}

/// display the view
pub fn display(view: &View, file_name: &Option<String>) {
    let height = view.height;
    let width = view.width;
    for line in 0..height {
        line_number(line);
        print!(
            "{}{}",
            cursor::Goto(line_number_width + 1, (line + 1) as u16),
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
