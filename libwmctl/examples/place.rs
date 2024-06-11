use libwmctl::prelude::*;

// Resize active window to half the screen size then position it to the right
fn main() {
    //let win = active();
    let win = first_by_class("alacritty").unwrap();
    win.shape(Shape::Large).pos(Position::TopRight).place().unwrap();

    //let win = first_by_class("alacritty").unwrap();
    //win.pos(Position::Bottom).place().unwrap();
    //win.pos(Position::BottomCenter).place().unwrap();
    //win.pos(Position::BottomLeft).place().unwrap();
    //win.pos(Position::BottomRight).place().unwrap();
    //win.pos(Position::Center).place().unwrap();
    //win.pos(Position::Left).place().unwrap();
    //win.pos(Position::LeftCenter).place().unwrap();
    //win.pos(Position::Right).place().unwrap();
    //win.pos(Position::RightCenter).place().unwrap();
    //win.pos(Position::Static(0, 0)).place().unwrap();
    //win.pos(Position::Top).place().unwrap();
    //win.pos(Position::TopCenter).place().unwrap();
    //win.pos(Position::TopLeft).place().unwrap();
    //win.pos(Position::TopRight).place().unwrap();
}
