// use rdev::{Event, listen};
use log::{debug, error};
use rdev::{Key, listen};
use tokio::sync::mpsc::{self, Receiver};
// use std::sync::mpsc::{self, Receiver};

pub fn spawn_keylogger() -> Receiver<Key> {
    let (tx, rx) = mpsc::channel(256);
    std::thread::spawn(move || {
        if let Err(err) = listen(move |event| {
            if let rdev::EventType::KeyPress(key) = event.event_type {
                debug!("keystroke detected: {:?}", key);
                let _ = tx.try_send(key);
            }
        }) {
            error!("rdev listener error: {:?}", err);
        }
    });
    rx
}
