use libwmctl::prelude::*;

// Explicit resize of the active window
fn main() {
    window(None).shape(Shape::Static(1200, 800)).pos(Position::Static(100, 100)).place().unwrap();
}
