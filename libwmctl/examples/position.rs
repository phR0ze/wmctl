use libwmctl::prelude::*;

// Move the active window
fn main() {
    // Position the active window on the left
    window(None).pos(Position::Left).place().unwrap();
}
