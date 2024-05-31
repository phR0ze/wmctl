use libwmctl::prelude::*;

// Resize active window to half the screen size then position it to the right
fn main() {
    //active().shape(Shape::Halfw).pos(Position::Right).place().unwrap();
    //window(104857608).shape(Shape::Halfw).pos(Position::Right).place().unwrap();
    //window(104857608).pos(Position::Static(0, 0)).place().unwrap();
    window(104857608).shape(Shape::Static(1272, 1388)).place().unwrap();
}
