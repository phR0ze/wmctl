use libwmctl::prelude::*;

// Resize active window to half the screen size then position it to the right
fn main() {
    window(None).shape(Shape::Halfw).pos(Position::Right).place().unwrap();
}
