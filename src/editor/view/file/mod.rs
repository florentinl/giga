//! # In-memory representation of the file being edited
//!
//! The File structure is the in-memory representation of the full file being edited with syntax highlighting.
//! It is a vector of lines, each line being a vector of ColorChar (a char and its associated color). There are
//! two types of operations on the File:
//! - Read operations: they are used to display the file on the screen
//! - Write operations: they are used to modify the file -> Trigger a recolorization of the file
mod color;

use color::{ColorChar, Colorizer};

/// In-memory representation of a syntax-highlighted file
pub struct File {
    /// The content of the file
    content: Vec<Vec<ColorChar>>,
    /// The colorizer used to perform syntax highlighting on the file
    colorizer: Colorizer,
}

impl File {
    pub fn new(extension: &str) -> Self {
        Self {
            content: vec![vec![]],
            colorizer: Colorizer::new(extension),
        }
    }

    /// Create a File abstraction from a string
    pub fn from_string(str: &str, extension: &str) -> Self {
        // Replace tabs with 4 spaces
        let str = str.replace('\t', "    ");
        let mut colorizer = Colorizer::new(extension);
        let content: Vec<Vec<ColorChar>> = colorizer.colorize_string(&str);
        Self { content, colorizer }
    }

    /// Recolorize the file
    pub fn recolorize(&mut self) {
        let content_as_string = self.to_string();
        self.content = self.colorizer.colorize_string(&content_as_string);
    }

    /// Get the nth line of the file
    pub fn get_line(&self, index: usize) -> Option<Vec<ColorChar>> {
        self.content.get(index).cloned()
    }

    /// Get the number of lines in the file
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Insert a char at the given position in the file
    /// - line >= len || col > line_len: do nothing
    /// - else insert the byte at the given position
    pub fn insert(&mut self, line: usize, col: usize, c: char) {
        match self.content.get_mut(line) {
            None => {}
            Some(line) => {
                if col > line.len() {
                    return;
                }
                let cc = ColorChar {
                    char: c,
                    color: termion::color::Rgb(0, 0, 0),
                };
                line.insert(col, cc);
                self.recolorize();
            }
        }
    }

    /// Delete a char at the given position
    /// - col == 0: join the line with the previous one (except if it's the first line)
    /// - 0 < col <= line_len: delete the byte at the given position
    /// - col > line_len: do nothing
    pub fn delete(&mut self, line: usize, col: usize) {
        if line >= self.content.len() {
            return;
        }

        let line_len = self.content[line].len();
        if col == 0 {
            if line > 0 {
                let prev_line = self.content.remove(line);
                if let Some(line) = self.content.get_mut(line - 1) {
                    line.extend(prev_line);
                    self.recolorize();
                }
            }
        } else if col <= line_len {
            self.content[line].remove(col - 1);
            self.recolorize();
        }
    }

    /// Split a line at the given position
    /// - line >= len: do nothing
    /// - col > line_len: do nothing
    /// - else split the line at the given position
    pub fn split_line(&mut self, line: usize, col: usize) {
        match self.content.get_mut(line) {
            None => {}
            Some(vec) => {
                if col > vec.len() {
                    return;
                }
                let new_line = vec.split_off(col);
                self.content.insert(line + 1, new_line);
                self.recolorize();
            }
        }
    }

    pub fn delete_line(&mut self, line: usize) {
        if line >= self.content.len() {
            return;
        }
        if line == 0 && self.content.len() == 1 {
            self.content[0].clear();
            return;
        } else {
            self.content.remove(line);
        }
        self.recolorize();
    }
}

/// Implement the ToString trait for File (used for saving the file)
impl ToString for File {
    fn to_string(&self) -> String {
        self.content
            .iter()
            .map(|line| line.iter().map(|c| c.char).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string_to_colorchars(str: &str) -> Vec<ColorChar> {
        str.chars()
            .map(|c| ColorChar {
                char: c,
                color: termion::color::Rgb(192, 197, 206),
            })
            .collect()
    }

    #[test]
    fn file_new_empty() {
        let file = File::new("txt");
        assert_eq!(file.content.len(), 1);
    }

    #[test]
    fn file_from_string() {
        let file = File::from_string("Hello, World !", "txt");
        assert_eq!(file.content.len(), 1);
        assert_eq!(file.content[0], string_to_colorchars("Hello, World !"));
    }

    #[test]
    fn file_to_string() {
        let file = File::from_string("Hello, World !", "txt");
        assert_eq!(file.to_string(), "Hello, World !");
    }

    #[test]
    fn file_get_line() {
        let file = File::from_string("Hello, World !\n", "txt");
        assert_eq!(
            file.get_line(0),
            Some(string_to_colorchars("Hello, World !"))
        );
        assert_eq!(file.get_line(1), Some(string_to_colorchars("")));
        assert_eq!(file.get_line(2), None);
    }

    #[test]
    fn file_get_len() {
        let file = File::from_string("Hello, World !\n", "txt");
        assert_eq!(file.len(), 2);
    }

    #[test]
    fn file_insert() {
        let mut file = File::from_string("Hello, World !\n", "txt");
        file.insert(0, 0, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n");
        file.insert(1, 0, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n!");
        file.insert(1, 1, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n!!");

        // Out of bounds line
        file.insert(2, 0, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n!!");

        // Out of bounds col
        file.insert(1, 3, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n!!");
    }

    #[test]
    fn file_delete() {
        let mut file = File::from_string("HW\n", "txt");
        file.delete(0, 1);
        assert_eq!(file.to_string(), "W\n");
        file.delete(0, 1);
        assert_eq!(file.to_string(), "\n");
    }

    #[test]
    fn file_delete_out_of_bounds() {
        let mut file = File::from_string("HW\n", "txt");
        file.delete(1, 1);
        assert_eq!(file.to_string(), "HW\n");
    }

    #[test]
    fn file_delete_beginning_of_line() {
        let mut file = File::from_string("HW\nGuys !", "txt");
        file.delete(1, 0);
        assert_eq!(file.to_string(), "HWGuys !");
    }

    #[test]
    fn file_split_line() {
        let mut file = File::from_string("Hello, World !", "txt");
        file.split_line(0, 5);
        assert_eq!(file.to_string(), "Hello\n, World !");
    }

    #[test]
    fn file_split_line_out_of_bounds_line() {
        let mut file = File::from_string("Hello, World !", "txt");
        file.split_line(1, 5);
        assert_eq!(file.to_string(), "Hello, World !");
    }

    #[test]
    fn file_split_line_out_of_bounds_lcol() {
        let mut file = File::from_string("Hello, World !", "txt");
        file.split_line(0, 20);
        assert_eq!(file.to_string(), "Hello, World !");
    }

    #[test]
    fn file_from_sting_with_tabs() {
        let file = File::from_string("Hello,\tWorld !", "txt");
        assert_eq!(file.to_string(), "Hello,    World !");
    }
}
