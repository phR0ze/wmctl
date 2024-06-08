use libwmctl::prelude::*;

fn main() {
    let wm = info().unwrap();
    let win = active();
    let parent = win.parent().unwrap();

    let (px, py, pw, ph) = parent.visual_geometry().unwrap();
    let (x, y, w, h) = win.geometry().unwrap();
    let (vx, vy, vw, vh) = win.visual_geometry().unwrap();
    let b = win.borders().unwrap();
    let g = win.gtk_borders().unwrap();

    println!("Window Information");
    println!("-----------------------------------------------------------------------");
    println!("ID:           {}", win.id);
    println!("Parent:       {}", parent.id);
    println!("Parent Geom:  x: {}, y: {}, w: {}, h: {}", px, py, pw, ph);
    if parent.id != wm.root_win_id {
        let grand_parent = parent.parent().unwrap();
        println!(
            "Grand Parent: {} {}",
            grand_parent.id,
            if grand_parent.id == wm.root_win_id { "is root window" } else { "is not root window" }
        );
    }
    println!("Class:        {}", win.class().unwrap_or("".to_owned()));
    println!("PID:          {}", win.pid().unwrap_or(-1));
    println!("Name:         {}", win.name().unwrap_or("".to_owned()));
    println!("Type:         {}", win.kind().unwrap_or(Kind::Invalid));
    println!("Desktop:      {}", win.desktop().unwrap_or(-1));
    println!("Win Geom:     x: {}, y: {}, w: {}, h: {}", x, y, w, h);
    println!("Visual Geom:  x: {}, y: {}, w: {}, h: {}", vx, vy, vw, vh);
    println!("WM Borders:   l: {}, r: {}, t: {}, b: {}", b.l, b.r, b.t, b.b);
    println!("GTK Borders:  l: {}, r: {}, t: {}, b: {}", g.l, g.r, g.t, g.b);
    println!("State:        {:?}", win.state().unwrap_or(vec![]));
    println!("Mapped:       {}", win.mapped().unwrap());
}
