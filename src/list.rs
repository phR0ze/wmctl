use clap::ArgMatches;
use libwmctl::prelude::*;
use prettytable::{format, Cell, Row, Table};
use witcher::prelude::*;

/// Run the subcommand
///
/// ### Arguments
/// * `global` - the ArgMatches object for the global arguments
pub fn run(global: &ArgMatches) -> Result<()> {
    let matches = global.subcommand_matches("list").unwrap();
    windows(matches.is_present("all"))
}

// List all windows
pub fn windows(all: bool) -> Result<()> {
    let windows = libwmctl::windows(all).unwrap();
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
        Cell::new("PARENT"),
        Cell::new("TYPE"),
        Cell::new("STATE"),
        Cell::new("CLASS"),
        Cell::new("NAME"),
    ]));

    for win in windows.iter() {
        let (x, y, w, h) = win.visual_geometry().unwrap();
        let b = if win.is_gtk() { win.gtk_borders() } else { win.borders() };
        table.add_row(Row::new(vec![
            Cell::new(&win.id.to_string()),
            Cell::new(&format!("{:>2}", win.desktop().unwrap())),
            Cell::new(&win.pid().unwrap_or(-1).to_string()),
            Cell::new(&x.to_string()),
            Cell::new(&y.to_string()),
            Cell::new(&w.to_string()),
            Cell::new(&h.to_string()),
            Cell::new(&format!("L{},R{},T{},B{}", b.l, b.r, b.t, b.b)),
            Cell::new(&format!("{}", win.parent().unwrap().id)),
            Cell::new(&win.kind().unwrap_or(Kind::Invalid).to_string()),
            Cell::new(&format!("{:?}", win.state().unwrap_or(vec![]))),
            Cell::new(&win.class().unwrap_or("".to_owned())),
            Cell::new(&win.name().unwrap_or("".to_owned())),
        ]));
    }
    table.printstd();

    Ok(())
}
