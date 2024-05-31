use libwmctl::prelude::*;

// Shape the active window as half the screen space and position left
fn main() {
    active().shape(Shape::Halfw).pos(Position::Left).place().unwrap();
}
