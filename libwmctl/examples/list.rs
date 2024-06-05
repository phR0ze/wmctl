use libwmctl::prelude::*;
use prettytable::{format, Cell, Row, Table};

// List all windows
fn main() {
    let windows = windows(false).unwrap();
    let mut table = Table::new();
    table.set_format(format::FormatBuilder::new().padding(1, 1).build());
    table.set_titles(Row::new(vec![
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

    for win in windows.iter() {
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
            Cell::new(&win.kind().unwrap_or(Kind::Invalid).to_string()),
            Cell::new(&format!("{:?}", win.state().unwrap_or(vec![State::Invalid]))),
            Cell::new(&win.class().unwrap_or("".to_owned())),
            Cell::new(&win.name().unwrap_or("".to_owned())),
        ]));
    }
    table.printstd();
}
