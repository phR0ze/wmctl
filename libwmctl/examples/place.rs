use libwmctl::prelude::*;

// Resize and move the active window
fn main() {
    WinOpt::new(None).shape(Shape::Halfw).pos(Position::Right).place().unwrap();
}
