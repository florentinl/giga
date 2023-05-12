# GIGA

A HeavyWeight text editor written in Rust

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