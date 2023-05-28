# Giga

## un editeur de texte écrit en Rust
Tout le code est disponible sur GitHub à https://github.com/florentinl/giga

---
layout: center
---

# Avant toute chose, une démonstration

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

# La structure de l'éditeur (simplifiée)

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
