---
layout: center
---

# Optimiser le rafraichissement

---
layout: center
---

au lieu d'appeler `draw_tui()` à chaque fois, on peut appeler lui passer un enum

```rust
pub enum RefreshOrder {
    /// Terminate the editor
    Terminate,
    /// No need to refresh the screen
    None,
    /// Refresh the cursor position
    CursorPos,
    /// Refresh the given lines
    Lines(HashSet<usize>),
    /// Refresh the status bar
    StatusBar,
    /// Refresh the whole screen
    AllLines,
}
```

On affiche donc seulement ce qui a changé

---
layout: center
---

## Encore plus précisemment

On peut réduire le nombre de caractères à afficher

- On ne clear pas avant d'écrire
- On cache le curseur avant d'écire (Flickering)

---
layout: center
---
## Une autre démonstration

```sh
giga README.md
```
