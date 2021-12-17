use libwmctl::prelude::*;

// Move the active window
fn main() {
    WinOpt::new(None).pos(WinPosition::Left).place().unwrap();
}