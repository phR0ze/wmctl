use libwmctl::prelude::*;

// Resize active window to half the screen size then position it to the right
fn main() {
    //window(104857608).shape(Shape::Halfw).pos(Position::Right).place().unwrap();
    //window(104857608).pos(Position::Static(0, 0)).place().unwrap();
    //let win = active();
    let win = first_by_class("firefox").unwrap();
    win.shape(Shape::Halfw).pos(Position::Right).place().unwrap();
    //win.shape(Shape::Static(1272, 1388)).place().unwrap();
}
