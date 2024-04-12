#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

//! ## Internals
//!
//! Open the documentation for the `editor` module to see how the editor works.
//! ```
//! cargo doc --open
//! ```
//!
//! The doc is also available as a [Github page](https://florentinl.github.io/giga/).

mod editor;
use crate::editor::tui;

use editor::Editor;

fn usage(progname: Option<&String>) {
    let name = match progname {
        Some(str) => str.clone(),
        None => "giga".to_string(),
    };
    println!("Usage: {} [file]", name);
    std::process::exit(1);
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 2 {
        let progname = args.get(0);
        usage(progname)
    }

    // Optional file to edit
    let file: Option<&str> = args.get(1).map(|s| s.as_str());
    let mut terminal = tui::init().unwrap();
    let mut editor = match file {
        // Try to open the file, if it doesn't exist, create a new one
        Some(path) => Editor::open(path),
        // If no file is provided, create a new one with a default name
        None => Editor::open("./Newfile"),
    };
    editor.run(&mut terminal).unwrap();
    tui::restore().unwrap();
}
