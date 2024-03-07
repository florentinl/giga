//! # Intercepting terminal resize events
//!
//! To intercept terminal resize events, we need to register a channel to listen to the SIGWINCH.
//! To do so, we use the libc crate to register a C function as a signal handler.
//!
//! This is a bit hacky but it works. It is the only part of the code which is `unsafe`.

use std::sync::mpsc::Sender;

use super::RefreshOrder;

// We have to use a global variable because the signal handler has to be a C function
static mut TX: Option<Sender<RefreshOrder>> = None;

// Using libc spawn a thread to intercept the SIGWINCH signal and send it through a channel
// given in parameter to the function.
pub fn init_resize_listener(tx: Sender<RefreshOrder>) {
    unsafe {
        TX = Some(tx);
    }
    unsafe {
        libc::signal(libc::SIGWINCH, resize_handler as libc::sighandler_t);
    }
}

// This function is called when the SIGWINCH signal is intercepted
unsafe extern "C" fn resize_handler(_: libc::c_int) {
    // Send the signal through the channel
    // Note: the send() function will fail if the receiver is not listening anymore
    //       (the receiver is the terminal drawer)
    let _ = TX.as_ref().unwrap().send(RefreshOrder::Resize);
}
