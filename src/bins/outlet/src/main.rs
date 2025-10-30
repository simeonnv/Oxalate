use oxalate_keylogger::spawn_keylogger;

fn main() {
    let rx = spawn_keylogger();

    println!("Hello, world!");
}
