use libwmctl::prelude::*;

fn main() {
    let wm = info().unwrap();
    //let win = active();
    let win = first_by_class("firefox").unwrap();
    let parent = win.parent().unwrap();
    let grand_parent = parent.parent().unwrap();

    let (px, py, pw, ph) = parent.geometry().unwrap_or((0, 0, 0, 0));
    let (x, y, w, h) = win.geometry().unwrap_or((0, 0, 0, 0));
    let (l, r, t, b) = win.borders().unwrap_or((0, 0, 0, 0));
    let (gl, gr, gt, gb) = win.gtk_borders().unwrap_or((0, 0, 0, 0));

    println!("Window Information");
    println!("-----------------------------------------------------------------------");
    println!("ID:           {}", win.id);
    println!("Parent:       {}", parent.id);
    println!("Parent Geom:  x: {}, y: {}, w: {}, h: {}", px, py, pw, ph);
    println!(
        "Grand Parent: {} {}",
        grand_parent.id,
        if grand_parent.id == wm.root_win_id { "is root window" } else { "is not root window" }
    );
    println!("Class:        {}", win.class().unwrap_or("".to_owned()));
    println!("PID:          {}", win.pid().unwrap_or(-1));
    println!("Name:         {}", win.name().unwrap_or("".to_owned()));
    println!("Type:         {}", win.kind().unwrap_or(Kind::Invalid));
    println!("Desktop:      {}", win.desktop().unwrap_or(-1));
    println!("Geometry:     x: {}, y: {}, w: {}, h: {}", x, y, w, h);
    println!("Borders:      l: {}, r: {}, t: {}, b: {}", l, r, t, b);
    println!("GTK Borders:  l: {}, r: {}, t: {}, b: {}", gl, gr, gt, gb);
    println!("State:        {:?}", win.state().unwrap_or(vec![]));
    println!("Mapped:       {}", win.mapped().unwrap());
}
