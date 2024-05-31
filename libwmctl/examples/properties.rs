use libwmctl::prelude::*;

// Get all window properties
fn main() {
    active().properties().unwrap();
}
