use std::sync::mpsc::Sender;

static mut TX: Option<Sender<()>> = None;

// Using libc spawn a thread to intercept the SIGWINCH signal and send it through a channel
// given in parameter to the function.
pub fn init_resize_listener(tx: Sender<()>) {
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
    let _ = TX.as_ref().unwrap().send(());
}
