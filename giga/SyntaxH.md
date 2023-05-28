---
layout: center
---

# Coloration Syntaxique

---
layout: center
---

## Qu'est ce qu'un parseur syntaxique ?

Un parseur syntaxique est un programme qui analyse un texte et le transforme en une structure de données.

On utilise [Syntect](https://github.com/trishume/syntect) qui integre un parseur syntaxique pour plus de 100 langages.

---
layout: center
---

## On modifie un petit peu la structure

```text {11}
$ tree
.
├── editor
│   ├── command.rs
│   ├── mod.rs
│   ├── terminal
│   │   ├── mod.rs       
│   │   └── termion.rs
│   └── view
│       ├── file
│       │   ├── colors.rs ← Initialise le parseur (Syntect) et gère les courleurs
│       │   └── mod.rs   
│       └── mod.rs      
└── main.rs              
```

---
layout: center
---

## Une dernière démonstration

```sh
giga src/main.rs
```

---
layout: center
---

## Optimisations ?

On pourrait utiliser un thread séparé pour le parseur syntaxique. Qui se chargerait de mettre à jour les couleurs de chaque caractère du fichier.
