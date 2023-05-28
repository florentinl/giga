---
layout: full
---

# Coloration Syntaxique

---
layout: full
---

# Qu'est ce qu'un parseur syntaxique ?

Un parseur syntaxique est un programme qui analyse un texte et le transforme en une structure de données.

On utilise [Syntect](https://github.com/trishume/syntect) qui integre un parseur syntaxique pour plus de 100 langages.

---
layout: full
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
layout: full
---

# Une dernière démonstration

```sh
giga src/main.rs
```

---
layout: full
---

# Optimisations ?

On pourrait utiliser un thread séparé pour le parseur syntaxique. Qui se chargerait de mettre à jour les couleurs de chaque caractère du fichier.
