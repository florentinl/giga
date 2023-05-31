---
layout: center
---

# Coloration Syntaxique

---
layout: center
---

# Qu'est ce qu'un parseur syntaxique ?

Un parseur syntaxique est un programme qui analyse un texte et le transforme en une structure de données.

On utilise [Syntect](https://github.com/trishume/syntect) qui integre un parseur syntaxique pour plus de 100 langages.

---
layout: center
---

# On modifie un petit peu la structure

```text {14}
$ tree

.
├── editor
│   ├── command.rs
│   ├── git.rs
│   ├── mod.rs
│   ├── signal.rs
│   ├── terminal
│   │   ├── mod.rs
│   │   └── termion.rs
│   └── view
│       ├── file
│       │   ├── color.rs ← Initialise le parseur (Syntect) et gère les courleurs
│       │   └── mod.rs
│       └── mod.rs
└── main.rs
```

---
layout: center
---

# On ajoute une structure pour stocker les couleurs

```rust
pub struct File {
    /// The content of the file
    content: Vec<Vec<ColorChar>>,
    /// The colorizer used to perform syntax highlighting on the file
    colorizer: Colorizer,
}
```

```rust
pub struct ColorChar {
    pub char: char,
    pub color: termion::color::Rgb,
}

pub struct Colorizer {
    ps: SyntaxSet, // Parser from syntect
    ts: ThemeSet,  // ThemeSet from syntect
    extension: String,
}
```
---
layout: center
---

# À chaque changement de fichier on parse
Exemple avec insert
```rust{13}
    pub fn insert(&mut self, line: usize, col: usize, c: char) {
        match self.content.get_mut(line) {
            None => {}
            Some(line) => {
                if col > line.len() {
                    return;
                }
                let cc = ColorChar {
                    char: c,
                    color: termion::color::Rgb(0, 0, 0),
                };
                line.insert(col, cc);
                self.recolorize();
            }
        }
    }
```

---
layout: center
---

# Une dernière démonstration

```sh
giga src/main.rs
```

---
layout: center
---

# Optimisations ?

On pourrait utiliser un thread séparé pour le parseur syntaxique. Qui se chargerait de mettre à jour les couleurs de chaque caractère du fichier.
