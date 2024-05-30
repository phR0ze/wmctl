use libwmctl::prelude::*;

// Get all window properties
fn main() {
    window(None).properties().unwrap();
}
