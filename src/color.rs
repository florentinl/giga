use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

#[derive(Clone, PartialEq, Debug)]
pub struct ColorChar {
    pub char: char,
    pub color: termion::color::Rgb,
}

pub struct Colorizer {
    ps: SyntaxSet,
    ts: ThemeSet,
    extension: String,
}

impl Colorizer {
    pub fn new(extension: &str) -> Self {
        Self {
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
            extension: String::from(extension),
        }
    }

    fn str_to_styled_lines(&mut self, str: &str) -> Vec<Vec<(Style, String)>> {
        let syntax = self
            .ps
            .find_syntax_by_extension(&self.extension)
            .unwrap_or(self.ps.find_syntax_plain_text());
        let mut h = HighlightLines::new(syntax, &self.ts.themes["base16-ocean.dark"]);
        let mut lines = Vec::new();
        for line in LinesWithEndings::from(str) {
            let mut styled_line = Vec::new();
            let mut start = 0;
            for (style, substr) in h.highlight_line(line, &self.ps).unwrap() {
                styled_line.push((style, String::from(&line[start..start + substr.len()])));
                start += substr.len();
            }
            lines.push(styled_line);
        }
        // If the last char is a newline, add an empty line
        if str.chars().last().unwrap() == '\n' {
            lines.push(Vec::new());
        }
        lines
    }

    fn styled_lines_to_colorchars(
        &mut self,
        styled_lines: Vec<Vec<(Style, String)>>,
    ) -> Vec<Vec<ColorChar>> {
        let mut colorchars = Vec::new();
        for line in styled_lines {
            let mut color_line = Vec::new();
            for (style, substr) in line {
                let color =
                    termion::color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                for char in substr.chars().filter(|c| *c != '\n') {
                    color_line.push(ColorChar { char, color });
                }
            }
            colorchars.push(color_line);
        }
        colorchars
    }
    pub fn colorize_string(&mut self, s: &str) -> Vec<Vec<ColorChar>> {
        let styled_lines = self.str_to_styled_lines(s);
        self.styled_lines_to_colorchars(styled_lines)
    }
}
