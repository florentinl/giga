# GIGA

A simple modal based text editor written in Rust. It has no ambition at all, it is merely a project to learn Rust and how to build a text editor.

## User Interface

![Giga](https://raw.githubusercontent.com/florentinl/giga/main/img/video.gif)

## Installation

If you have cargo installed, you can install Giga by running:

```Bash
cargo install --git https://github.com/florentinl/giga.git
```

If you don't have cargo installed, you can download the binary from the [releases page](https://github.com/florentinl/giga/releases) or build it yourself.

```Bash
git clone https://github.com/florentinl/giga.git
cd giga
cargo build --release
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

It will create a new file called NewFile, duh.

## Mode

Giga is a modal based test editor. You have three modes:

- NORMAL
- INSERT
- RENAME

To toggle modes:

- in **NORMAL** -> `i` -> **INSERT**
- in **NORMAL** -> `R` -> **RENAME**
- in **INSERT** -> `Esc`-> **NORMAL**
- in **RENAME** -> `Enter` -> **NORMAL**

## TODO

A never ending list of things, but who cares, it's just a project to learn Rust.
