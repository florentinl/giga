---
layout: center
---

# Ne pas tout redessiner à chaque fois

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

---
layout: center
---

# Encore plus précisemment

On peut réduire le nombre de caractères à afficher

- On ne clear pas avant d'écrire
- On cache le curseur avant d'écire (Flickering)

---
layout: center
---
# Une autre démonstration
