use libwmctl::prelude::*;

// Shape the active window as half the screen space and position left
fn main() {
    //active().shape(Shape::Halfw).pos(Position::Left).place().unwrap();

    // Shape firefox
    first_by_class("firefox").and_then(|x| x.shape(Shape::Halfw).place().ok());
}
