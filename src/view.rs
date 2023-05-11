use crate::file::File;

/// The View struct represents the actual portion of the File being displayed.
pub struct View {
    /// The file being displayed
    file: File,
    /// The line number of the first line being displayed
    start_line: usize,
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

    pub fn resize(&mut self, height: usize, width: usize) {
        self.height = height;
        self.width = width;
    }

    pub fn get_line(&self, index: usize) -> String {
        let line = self
            .file
            .get_line(index + self.start_line)
            .unwrap_or_default();
        let start = self.start_col.min(line.len());
        let end = (self.start_col + self.width).min(line.len());
        String::from_utf8_lossy(&line[start..end]).to_string()
    }

    /// Navigate the cursor by a given amount and eventually scroll the view
    /// if the cursor is out of bounds of the file, it will be moved to the
    /// closest valid position instead.
    pub fn navigate(&mut self, dx: isize, dy: isize) {
        // We move onto the new line
        self.navigate_y(dy);

        // We move onto the new column
        self.navigate_x(dx)
    }
    /// Navigate along the y axis and eventually scroll the view
    fn navigate_y(&mut self, dy: isize) {
        let (_, y) = self.cursor;
        let file_height = self.file.len() as isize;
        let view_height = self.height as isize;
        let start = self.start_line as isize;
        let mut new_y = y as isize + dy;

        if new_y < 0 {
            self.start_line = (start + new_y).max(0) as usize;
            new_y = 0;
        } else if new_y >= view_height {
            self.start_line = (start + new_y - view_height + 1)
                .min(file_height - view_height)
                .max(0) as usize;
            new_y = (view_height - 1).min(file_height - 1);
        } else {
            new_y = new_y.max(0).min(file_height - 1);
        }
        self.cursor.1 = new_y as usize;
    }

    /// Navigate along the x axis and eventually scroll the view
    fn navigate_x(&mut self, dx: isize) {
        let line_len = self
            .file
            .get_line(self.cursor.1 + self.start_line)
            .unwrap_or_default()
            .len() as isize;
        let left = self.start_col as isize;
        let width = self.width as isize;

        // calculate the absolute position of the cursor
        let abs_x = (self.cursor.0 as isize + dx + left).max(0).min(line_len);
        let rel_x = abs_x - left;

        if rel_x < 0 {
            self.start_col = (left + rel_x).max(0) as usize;
            self.cursor.0 = 0;
        } else if rel_x >= width {
            self.start_col = ((left + rel_x).min(line_len) - width + 1).max(0) as usize;
            self.cursor.0 = (width - 1) as usize;
        } else {
            self.cursor.0 = rel_x as usize;
        }
    }

    pub fn insert(&mut self, c: char) {
        match c {
            '\n' => {
                let (x, y) = self.cursor;
                self.file.split_line(y, x);
                self.navigate(-(x as isize), 1);
            }
            '\t' => {
                let (x, y) = self.cursor;
                for _ in 0..4 {
                    self.file.insert(y, x, b' '); // TODO: replace with tab
                }
                self.navigate(4, 0);
            }
            _ => {
                let (x, y) = self.cursor;
                self.file.insert(y, x, c as u8);
                self.navigate(1, 0);
            }
        }
    }

    pub fn delete(&mut self) {
        let (x, y) = self.cursor;
        match (x, y) {
            (0, 0) => return,
            (0, _) => {
                // get previous line length
                let line = self.file.get_line(y - 1).unwrap_or_default();
                let dx = line.len() as isize;
                let dy = -1;
                self.file.join_line(y);
                self.navigate(dx, dy)
            }
            (_, _) => {
                self.file.delete(y, x - 1);
                self.navigate(-1, 0);
            }
        }
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
        let view = View::new(File::from_bytes(b"Hello, World !\n"), 1, 10);
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
        let view = View::new(File::from_bytes(b"Hello, World !\n"), 1, 10);
        assert_eq!(view.get_line(0), "Hello, Wor");
    }

    #[test]
    fn view_navigate() {
        let mut view = View::new(
            File::from_bytes(b"Hello, World !\nWelcome to the moon!"),
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
            File::from_bytes(b"Hello, World !\nWelcome to the moon!"),
            2,
            100,
        );
        view.navigate(30, 0);
        assert_eq!(view.cursor, (14, 0));
    }

    #[test]
    fn view_navigate_go_to_eof() {
        let mut view = View::new(
            File::from_bytes(b"Hello, World !\nWelcome to the moon!"),
            3,
            100,
        );
        view.navigate(0, 30);
        assert_eq!(view.cursor, (0, 1));
    }

    #[test]
    fn view_navigate_scroll_y() {
        let mut view = View::new(
            File::from_bytes(b"Hello, World !\nWelcome to the moon!"),
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
            File::from_bytes(b"Hello, World !\nWelcome to the moon!"),
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
}
