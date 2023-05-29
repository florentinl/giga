---
layout: full
---

# Les signaux en Rust

Lorsque l'utilisateur redimensionne la fenêtre, le programme reçoit un signal
<kbd style="font-size: 1.5em">SIGWINCH</kbd>.

J'ai pas trouvé... → du coup on fait du C.


```rust {all|6-9|14-17|all}
static mut TX: Option<Sender<()>> = None;

pub fn init_resize_listener(tx: Sender<()>) {
    unsafe {
        TX = Some(tx);
        libc::signal(
            libc::SIGWINCH,
            resize_handler as libc::sighandler_t
        );
    }
}

// Handler called on SIGWINCH
unsafe extern "C" fn resize_handler(_: libc::c_int) {
    TX.as_ref().unwrap().send(());
}
```

---
layout: full
---

# Intégration dans Giga


<div style="
    width: 100%;
    margin-top: 10%;
    margin-left: 6%;
    ">

<Transform
    :scale=1.3
    >

```mermaid {themeVariables: {nodeBorder: '#885921'}}
stateDiagram
direction TB
Main/Input --> Tui/Drawing : RefreshOrder
Main/Input --> Main/Input : ReadInput
ResizeListener --> Tui/Drawing : SizeChanged
Git --> Tui/Drawing : DiffChanged
Git --> Git : RecomputeDiff
```

</Transform>

</div>

---
layout: center
---

# Encore une démo
