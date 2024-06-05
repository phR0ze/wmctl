use libwmctl::prelude::*;

// Get all window properties for the first window that matches the given class
fn main() {
    first_by_class("Evince").and_then(|x| x.properties().ok()).unwrap();
}
