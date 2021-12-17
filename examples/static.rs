use libwmctl::prelude::*;

// Explicit resize of the active window
fn main() {
    WinOpt::new(None).size(1200, 800).location(0, 0).place().unwrap();
}