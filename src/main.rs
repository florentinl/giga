mod editor;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 2 {
        println!("Usage: giga [file]")
    }

    // Optional file to edit
    let file: Option<&str> = args.get(1).map(|s| s.as_str());
}
