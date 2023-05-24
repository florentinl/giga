use crate::file::File;

/// The View struct represents the actual portion of the File being displayed.
pub struct View {
    /// The file being displayed
    file: File,
    /// The line number of the first line being displayed
    pub start_line: usize,
    /// The column number of the first column being displayed
    start_col: usize,
    /// The number of lines being displayed
    pub height: usize,
    /// The number of columns being displayed
    pub width: usize,
    /// The position of the cursor in the view
    pub cursor: (usize, usize),
}

impl View {
    /// Create a new View
    pub fn new(file: File, height: usize, width: usize) -> Self {
        Self {
            file,
            start_line: 0,
            start_col: 0,
            height,
            width,
            cursor: (0, 0),
        }
    }

    /// Resize the view
    pub fn resize(&mut self, height: usize, width: usize) {
        self.height = height;
        self.width = width;
    }

    /// Get the line at the given index in the view
    pub fn get_line(&self, index: usize) -> String {
        let line = self
            .file
            .get_line(index + self.start_line)
            .unwrap_or_default();
        let start = self.start_col.min(line.len());
        let end = (self.start_col + self.width).min(line.len());
        String::from(&line[start..end].iter().collect::<String>())
    }

    /// Navigate the cursor by a given amount and eventually scroll the view
    /// if the cursor is out of bounds of the file, it will be moved to the
    /// closest valid position instead.
    pub fn navigate(&mut self, dx: isize, dy: isize) -> bool {
        // We move onto the new line
        let has_scrolled_on_y = self.navigate_y(dy);

        // We move onto the new column
        let has_scrolled_on_x = self.navigate_x(dx);

        has_scrolled_on_x || has_scrolled_on_y
    }
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
            .get_line(y + self.start_line)
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

    /// # Insert a character at the cursor position
    /// This function will insert a character at the cursor position and move
    /// the cursor to the right.
    pub fn insert(&mut self, c: char) -> bool {
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
    pub fn insert_new_line(&mut self) -> bool {
        let (rel_x, rel_y) = self.cursor;
        // Calculate the absolute position of the cursor in the file
        let (x, y) = (rel_x + self.start_col, rel_y + self.start_line);
        // Split the line at the cursor position
        self.file.split_line(y, x);
        // Navigate the cursor
        self.navigate(-(x as isize), 1)
    }

    pub fn delete(&mut self) -> bool {
        let scroll: bool;
        let (rel_x, rel_y) = self.cursor;
        // Calculate the absolute position of the cursor in the file
        let (x, y) = (rel_x + self.start_col, rel_y + self.start_line);

        // Get previous line length in case we need to go to the end of it
        let prev_line_len = self
            .file
            .get_line(y.saturating_sub(1))
            .unwrap_or_default()
            .len();

        // Delete the character at the cursor
        self.file.delete(y, x);

        // Navigate the cursor
        if x > 0 {
            scroll = self.navigate(-1, 0);
        } else {
            scroll = self.navigate(prev_line_len as isize, -1);
        }
        scroll
    }

    pub fn dump_file(&self) -> String {
        self.file.to_string()
    }
}

impl ToString for View {
    fn to_string(&self) -> String {
        let bottom = self
            .height
            .min(self.file.len().saturating_sub(self.start_line));

        (0..bottom)
            .map(|i| self.get_line(i))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_new() {
        let view = View::new(File::new(), 10, 10);
        assert_eq!(view.start_line, 0);
        assert_eq!(view.start_col, 0);
        assert_eq!(view.height, 10);
        assert_eq!(view.width, 10);
    }

    #[test]
    fn view_to_string() {
        let view = View::new(File::from_string("Hello, World !\n"), 1, 10);
        assert_eq!(view.to_string(), "Hello, Wor");
    }

    #[test]
    fn view_resize() {
        let mut view = View::new(File::new(), 10, 10);
        view.resize(20, 20);
        assert_eq!(view.height, 20);
        assert_eq!(view.width, 20);
    }

    #[test]
    fn view_get_line() {
        let view = View::new(File::from_string("Hello, World !\n"), 1, 10);
        assert_eq!(view.get_line(0), "Hello, Wor");
    }

    #[test]
    fn view_navigate() {
        let mut view = View::new(
            File::from_string("Hello, World !\nWelcome to the moon!"),
            2,
            10,
        );
        view.navigate(1, 1);
        assert_eq!(view.cursor, (1, 1));
        view.navigate(-1, -1);
        assert_eq!(view.cursor, (0, 0));
    }

    #[test]
    fn view_navigate_go_to_eol() {
        let mut view = View::new(
            File::from_string("Hello, World !\nWelcome to the moon!"),
            2,
            100,
        );
        view.navigate(30, 0);
        assert_eq!(view.cursor, (14, 0));
    }

    #[test]
    fn view_navigate_go_to_eof() {
        let mut view = View::new(
            File::from_string("Hello, World !\nWelcome to the moon!"),
            3,
            100,
        );
        view.navigate(0, 30);
        assert_eq!(view.cursor, (0, 1));
    }

    #[test]
    fn view_navigate_scroll_y() {
        let mut view = View::new(
            File::from_string("Hello, World !\nWelcome to the moon!"),
            1,
            100,
        );
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
        let mut view = View::new(
            File::from_string("Hello, World !\nWelcome to the moon!"),
            1,
            10,
        );
        view.navigate(9, 0);
        assert_eq!(view.cursor, (9, 0));
        assert_eq!(view.start_col, 0);
        view.navigate(1, 0);
        assert_eq!(view.cursor, (9, 0));
        assert_eq!(view.start_col, 1);
        view.navigate(20, 0);
        assert_eq!(view.get_line(0), ", World !");
        assert_eq!(view.cursor, (9, 0));
        assert_eq!(view.start_col, 5);
        view.navigate(-20, 0);
        assert_eq!(view.cursor, (0, 0));
        assert_eq!(view.start_col, 0);
    }

    #[test]
    fn view_insert() {
        let mut view = View::new(File::from_string("Hello, World !\n"), 1, 10);
        view.insert('a');
        assert_eq!(view.to_string(), "aHello, Wo");
        assert_eq!(view.cursor, (1, 0));
    }

    #[test]
    fn view_insert_non_ascii() {
        let mut view = View::new(File::from_string("Hello, World !\n"), 1, 10);
        view.insert('é');
        assert_eq!(view.to_string(), "éHello, Wo");
        assert_eq!(view.cursor, (1, 0));
    }

    #[test]
    fn view_insert_new_line() {
        let mut view = View::new(File::from_string("Hello, World !\n"), 10, 10);
        view.navigate(7, 0);
        view.insert_new_line();
        assert_eq!(view.dump_file(), "Hello, \nWorld !\n");
        assert_eq!(view.cursor, (0, 1));
    }

    #[test]
    fn view_delete() {
        let mut view = View::new(File::from_string("Hello, World !\n"), 1, 10);
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
