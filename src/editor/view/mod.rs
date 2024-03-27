//! # Portion of the file being displayed
//!
//! The view module represents the portion of the file being displayed on the screen. It
//! depends on the size of the terminal. And contains the cursor and scrolling logic. It
//! provides a cropped view of the file.

pub mod file;

use std::process::exit;
use std::{collections::HashMap, path};

use file::File;

use self::file::git::PatchType;
use self::file::EditorFile;

/// The View struct represents the actual portion of the File being displayed.
pub struct View {
    /// The file being displayed
    file: File,
    /// The line number of the first line being displayed
    pub start_line: usize,
    /// The column number of the first column being displayed
    pub start_col: usize,
    /// The number of lines being displayed
    pub height: usize,
    /// The number of columns being displayed
    pub width: usize,
    /// The position of the cursor in the view
    pub cursor: (usize, usize),
}

pub trait FileView {
    fn new(file_path: &str) -> Self;
    fn line(&self, index: usize) -> String;
    fn navigate(&mut self, dx: isize, dy: isize) -> bool;
    fn insert(&mut self, c: char) -> bool;
    fn insert_new_line(&mut self) -> bool;
    fn delete(&mut self) -> bool;
    fn delete_line(&mut self) -> bool;
    fn dump_file(&self) -> String;
    fn git_ref(&self) -> Option<String>;
    fn refresh_diff(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn diff(&self) -> Option<HashMap<usize, PatchType>>;
    fn file_path(&self) -> String;
    fn file_name(&self) -> String;
    fn file_dir(&self) -> String;
    fn set_file_name(&mut self, file_name: String);
}

impl Default for View {
    fn default() -> Self {
        Self {
            file: File::new("."),
            start_line: 0,
            start_col: 0,
            height: 0,
            width: 0,
            cursor: (0, 0),
        }
    }
}

impl From<String> for View {
    fn from(value: String) -> Self {
        Self {
            file: File::from_string(&value, "New file", "."),
            start_line: 0,
            start_col: 0,
            height: 0,
            width: 0,
            cursor: (0, 0),
        }
    }
}

impl FileView for View {
    fn new(file_path: &str) -> Self {
        let (file_dir, file_name, _) = split_path_name(file_path);
        let content_res = std::fs::read_to_string(file_path);
        let content;
        match content_res {
            Ok(c) => content = c,
            Err(_) => {
                eprintln!("Could not read file: {}", file_path);
                exit(1);
            }
        }
        let file = File::from_string(&content, &file_name, &file_dir);

        Self {
            file,
            start_line: 0,
            start_col: 0,
            height: 0,
            width: 0,
            cursor: (0, 0),
        }
    }

    /// Get the line at the given index in the view
    fn line(&self, index: usize) -> String {
        if let Some(line) = self.file.line(index + self.start_line) {
            let start = self.start_col;
            let end = (start + self.width).min(line.len());
            line[start..end].iter().collect()
        } else {
            String::new()
        }
    }

    /// Navigate the cursor by a given amount and eventually scroll the view
    /// if the cursor is out of bounds of the file, it will be moved to the
    /// closest valid position instead.
    fn navigate(&mut self, dx: isize, dy: isize) -> bool {
        // We move onto the new line
        let has_scrolled_on_y = self.navigate_y(dy);

        // We move onto the new column
        let has_scrolled_on_x = self.navigate_x(dx);

        has_scrolled_on_x || has_scrolled_on_y
    }
    /// # Insert a character at the cursor position
    /// This function will insert a character at the cursor position and move
    /// the cursor to the right.
    fn insert(&mut self, c: char) -> bool {
        let (rel_x, rel_y) = self.cursor;
        // Calculate the absolute position of the cursor in the file
        let (x, y) = (rel_x + self.start_col, rel_y + self.start_line);
        // Insert the character at the cursor position
        self.file.insert(y, x, c);
        self.navigate(1, 0)
    }

    /// # Insert a new line at the cursor position
    /// This function will split the line at the cursor position and move the
    /// cursor to the beginning of the new line.
    /// Example:
    /// ```text
    /// Hello, world!
    ///        ^ cursor is here
    /// ```
    /// ```text
    /// Hello,
    /// world!
    /// ^ cursor is here
    /// ```
    fn insert_new_line(&mut self) -> bool {
        let (rel_x, rel_y) = self.cursor;
        // Calculate the absolute position of the cursor in the file
        let (x, y) = (rel_x + self.start_col, rel_y + self.start_line);
        // Split the line at the cursor position
        self.file.split_line(y, x);
        // Navigate the cursor
        self.navigate(-(x as isize), 1)
    }

    fn delete(&mut self) -> bool {
        let (rel_x, rel_y) = self.cursor;
        // Calculate the absolute position of the cursor in the file
        let (x, y) = (rel_x + self.start_col, rel_y + self.start_line);

        // Get previous line length in case we need to go to the end of it
        let prev_line_len = self
            .file
            .line(y.saturating_sub(1))
            .unwrap_or_default()
            .len();

        // Delete the character at the cursor
        self.file.delete(y, x);

        // Navigate the cursor
        if x > 0 {
            self.navigate(-1, 0)
        } else {
            self.navigate(prev_line_len as isize, -1)
        }
    }

    fn delete_line(&mut self) -> bool {
        let (rel_x, rel_y) = self.cursor;
        // Calculate the absolute position of the cursor in the file
        let (x, y) = (rel_x + self.start_col, rel_y + self.start_line);

        // Delete the line at the cursor
        self.file.delete_line(y);

        // Navigate the cursor
        self.navigate(-(x as isize), 0)
    }

    /// Dump the content of the file to save it to disk
    fn dump_file(&self) -> String {
        self.file.to_string()
    }

    fn git_ref(&self) -> Option<String> {
        self.file.git_ref()
    }

    fn file_path(&self) -> String {
        self.file.file_dir.clone() + &self.file.file_name
    }

    fn set_file_name(&mut self, file_name: String) {
        self.file.file_name = file_name;
    }

    fn file_name(&self) -> String {
        self.file.file_name.clone()
    }

    fn file_dir(&self) -> String {
        self.file.file_dir.clone()
    }

    fn diff(&self) -> Option<HashMap<usize, PatchType>> {
        self.file.diff()
    }

    fn refresh_diff(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.file.refresh_diff()
    }
}

impl View {
    /// Navigate along the y axis and eventually scroll the view
    fn navigate_y(&mut self, dy: isize) -> bool {
        let (_, y) = self.cursor;
        let file_height = self.file.len() as isize;
        let view_height = self.height as isize;
        let top = self.start_line as isize;

        // calculate the absolute position of the cursor
        let abs_y = (y as isize)
            .saturating_add(dy)
            .saturating_add(top)
            .max(0)
            .min(file_height - 1);
        let rel_y = abs_y - top;

        if rel_y < 0 {
            self.start_line = (top + rel_y) as usize;
            self.cursor.1 = 0;
            true
        } else if rel_y >= view_height {
            self.start_line = (top + rel_y - view_height + 1)
                .min(file_height - view_height)
                .max(0) as usize;
            self.cursor.1 = (view_height - 1).min(file_height - 1) as usize;
            true
        } else {
            self.cursor.1 = rel_y as usize;
            false
        }
    }

    /// Navigate along the x axis and eventually scroll the view
    fn navigate_x(&mut self, dx: isize) -> bool {
        let (x, y) = self.cursor;
        let line_len = self
            .file
            .line(y + self.start_line)
            .unwrap_or_default()
            .len() as isize;
        let left = self.start_col as isize;
        let width = self.width as isize;

        // calculate the absolute position of the cursor
        let abs_x = (x as isize)
            .saturating_add(dx)
            .saturating_add(left)
            .max(0)
            .min(line_len);
        let rel_x = abs_x - left;

        if rel_x < 0 {
            self.start_col = (left + rel_x).max(0) as usize;
            self.cursor.0 = 0;
            true
        } else if rel_x >= width {
            self.start_col = ((left + rel_x).min(line_len) - width + 1).max(0) as usize;
            self.cursor.0 = (width - 1) as usize;
            true
        } else {
            self.cursor.0 = rel_x as usize;
            false
        }
    }
}

impl ToString for View {
    fn to_string(&self) -> String {
        let bottom = self
            .height
            .min(self.file.len().saturating_sub(self.start_line));

        (0..bottom)
            .map(|i| self.line(i))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn split_path_name(path: &str) -> (String, String, String) {
    let path = path::Path::new(path);
    let mut file_path = path.parent().unwrap().to_str().unwrap_or_default();
    if file_path.is_empty() {
        file_path = ".";
    }
    let file_name = path.file_name().unwrap().to_str().unwrap_or_default();
    let file_ext = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();

    (
        String::from(file_path) + "/",
        String::from(file_name),
        String::from(file_ext),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_to_string() {
        let mut view = View::from("Hello, World !\n".to_string());
        view.height = 1;
        view.width = 10;
        assert_eq!(view.to_string(), "Hello, Wor");
    }

    #[test]
    fn view_navigate() {
        let mut view = View::from("Hello, World !\nWelcome to the moon!".to_string());
        view.height = 2;
        view.width = 10;

        view.navigate(1, 1);
        assert_eq!(view.cursor, (1, 1));
        view.navigate(-1, -1);
        assert_eq!(view.cursor, (0, 0));
    }

    #[test]
    fn view_navigate_go_to_eol() {
        let mut view = View::from("Hello, World !\nWelcome to the moon!".to_string());
        view.height = 2;
        view.width = 100;

        view.navigate(30, 0);
        assert_eq!(view.cursor, (14, 0));
    }

    #[test]
    fn view_navigate_go_to_eof() {
        let mut view = View::from("Hello, World !\nWelcome to the moon!".to_string());
        view.height = 3;
        view.width = 100;

        view.navigate(0, 30);
        assert_eq!(view.cursor, (0, 1));
    }

    #[test]
    fn view_navigate_scroll_y() {
        let mut view = View::from("Hello, World !\nWelcome to the moon!".to_string());
        view.height = 1;
        view.width = 100;

        view.navigate(0, 1);
        assert_eq!(view.cursor, (0, 0));
        assert_eq!(view.start_line, 1);
        view.navigate(0, 1);
        assert_eq!(view.cursor, (0, 0));
        assert_eq!(view.start_line, 1);
        view.navigate(0, -1);
        assert_eq!(view.cursor, (0, 0));
        assert_eq!(view.start_line, 0);
        view.navigate(0, -1);
        assert_eq!(view.cursor, (0, 0));
        assert_eq!(view.start_line, 0);
    }

    #[test]
    fn view_navigate_scroll_x() {
        let mut view = View::from("Hello, World !\nWelcome to the moon!".to_string());
        view.height = 1;
        view.width = 10;

        view.navigate(9, 0);
        assert_eq!(view.cursor, (9, 0));
        assert_eq!(view.start_col, 0);
        view.navigate(1, 0);
        assert_eq!(view.cursor, (9, 0));
        assert_eq!(view.start_col, 1);
        view.navigate(20, 0);
        assert_eq!(view.line(0), ", World !");
        assert_eq!(view.cursor, (9, 0));
        assert_eq!(view.start_col, 5);
        view.navigate(-20, 0);
        assert_eq!(view.cursor, (0, 0));
        assert_eq!(view.start_col, 0);
    }

    #[test]
    fn view_insert() {
        let mut view = View::from("Hello, World !\n".to_string());
        view.height = 1;
        view.width = 10;
        view.insert('a');
        assert_eq!(view.to_string(), "aHello, Wo");
        assert_eq!(view.cursor, (1, 0));
    }

    #[test]
    fn view_insert_non_ascii() {
        let mut view = View::from("Hello, World !\n".to_string());
        view.height = 1;
        view.width = 10;
        view.insert('é');
        assert_eq!(view.to_string(), "éHello, Wo");
        assert_eq!(view.cursor, (1, 0));
    }

    #[test]
    fn view_insert_new_line() {
        let mut view = View::from("Hello, World !\n".to_string());
        view.height = 10;
        view.width = 10;
        view.navigate(7, 0);
        view.insert_new_line();
        assert_eq!(view.dump_file(), "Hello, \nWorld !\n");
        assert_eq!(view.cursor, (0, 1));
    }

    #[test]
    fn view_delete() {
        let mut view = View::from("Hello, World !\n".to_string());
        view.height = 1;
        view.width = 10;

        view.navigate(1, 0);
        view.delete();
        assert_eq!(view.to_string(), "ello, Worl");
        assert_eq!(view.cursor, (0, 0));
        // delete beginning of line
        view.navigate(0, 1);
        view.delete();
        assert_eq!(view.cursor, (9, 0));
        assert_eq!(view.to_string(), ", World !");
    }
}
