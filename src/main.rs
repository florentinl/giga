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

use editor::Editor;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 2 {
        println!("Usage: giga [file]");
        std::process::exit(1);
    }

    // Optional file to edit
    let file: Option<&str> = args.get(1).map(|s| s.as_str());

    let mut editor = match file {
        // Try to open the file, if it doesn't exist, create a new one
        Some(path) => Editor::open(path),
        // If no file is provided, create a new one with a default name
        None => Editor::open("./Newfile"),
    };
    editor.run();
}
