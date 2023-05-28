# Giga

## A simple editor written in rust
All code of our editor is available on [github](https://github.com/florentinl/giga)

---

# Why Rust ?

![cargo](https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg)

Rust is safe, fast and a modern system-programming language.

More, with cargo it's easy to :

- Build
- Test
- Run
- Publish
- Create Documentation

---
layout: center
---

# Let's run a demo

```sh
giga README.md
```

---
layout: center
---

# Modes

```mermaid
stateDiagram
direction TB
[*] --> NORMAL
NORMAL --> INSERT : i
INSERT --> NORMAL : Escape
NORMAL --> RENAME : R
RENAME --> NORMAL : Enter
NORMAL --> [*] : q
```

---
layout: center
---

# Structure (simplified)

```mermaid
flowchart
direction LR
  subgraph Editor
    subgraph View
        subgraph File
            content
        end
        navigate
    end
    subgraph Terminal
        content -->draw_tui
        navigate --> draw_tui
    end
  end
  KeyEvent --> navigate
  KeyEvent --> content
```
