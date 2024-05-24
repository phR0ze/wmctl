use libwmctl::prelude::*;
use prettytable::{Cell, Row, Table};

fn main() {
    let wm = window_manager().unwrap();
    let win = window(None);

    println!("X11 Information");
    println!("-----------------------------------------------------------------------");
    println!("Window Manager: {}", wm.name);
    println!("Compositing:    {}", wm.compositing);
    println!("Root Window:    {}", wm.root_win_id);
    println!("Work area:      {}x{}", wm.work_area.0, wm.work_area.1);
    println!("Screen Size:    {}x{}", wm.screen_size.0, wm.screen_size.1);
    println!("Desktops:       {}", wm.desktops);
    println!();

    println!("Active Window");
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new("DSK"),
        Cell::new("PID"),
        Cell::new("X"),
        Cell::new("Y"),
        Cell::new("W"),
        Cell::new("H"),
        Cell::new("BORDERS"),
        Cell::new("TYPE"),
        Cell::new("STATE"),
        Cell::new("CLASS"),
        Cell::new("NAME"),
    ]));

    let (x, y, w, h) = win.geometry().unwrap_or((0, 0, 0, 0));
    let (l, r, t, b) = win.borders().unwrap_or((0, 0, 0, 0));
    table.add_row(Row::new(vec![
        Cell::new(&win.id.to_string()),
        Cell::new(&format!("{:>2}", win.desktop().unwrap_or(-1))),
        Cell::new(&win.pid().unwrap_or(-1).to_string()),
        Cell::new(&x.to_string()),
        Cell::new(&y.to_string()),
        Cell::new(&w.to_string()),
        Cell::new(&h.to_string()),
        Cell::new(&format!("L{},R{},T{},B{}", l, r, t, b)),
        Cell::new(&win.kind().unwrap_or(WinKind::Invalid).to_string()),
        Cell::new(&format!("{:?}", win.state().unwrap_or(vec![WinState::Invalid]))),
        Cell::new(&win.class().unwrap_or("".to_owned())),
        Cell::new(&win.name().unwrap_or("".to_owned())),
    ]));
    table.printstd();
}
