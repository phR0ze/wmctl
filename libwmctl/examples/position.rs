use libwmctl::prelude::*;

// Move the active window
fn main() {
    // Position the active window on the left
    active().pos(Position::Left).place().unwrap();
}
