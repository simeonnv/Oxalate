// use rdev::{Event, listen};
use rdev::{Key, listen};
use std::sync::mpsc::{self, Receiver};

pub fn spawn_keylogger() -> Receiver<Key> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Err(err) = listen(move |event| {
            if let rdev::EventType::KeyPress(key) = event.event_type {
                tx.send(key).unwrap()
            }
        }) {
            eprintln!("rdev listener error: {:?}", err);
        }
    });
    rx
}
