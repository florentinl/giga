# GIGA

A HeavyWeight text editor written in Rust

> All this README.md file is written using giga !

## User Interface

![Giga](./img/video.gif)

## Installation

make sure you have cargo installed on your machine.

clone the repository:

```Bash
git clone https://github.com/florentinl/giga.git && cd giga
````

install the binary:

```Bash
cargo install --path .
```

## Editor

To start editing a file just write in your terminal:

```Bash
giga file.rs
```

You can also create a file and editing it:

```Bash
giga my_new_file.rs
```

If you enter

```Bash
giga
```

It will create a new file called NewFile.

## Mode

Giga is a modal based test editor. You have two modes:

- NORMAL
- INSERT

You can switch mode : press "i" in NORMAL to go to INSERT and "Esc" in INSERT to come back to NORMAL.
You can know the current mode by Looking to the status bar beneath the editor.

## TODO

As giga is under development, we need to fix these:

- [x] UTF-8 support
- [x] Rename a file
- [ ] Syntax Highlighting
- [x] Refresh only lines changed
- [x] Transform tabs in spaces
- [ ] Unit Testing 100%
- [ ] Documentation
