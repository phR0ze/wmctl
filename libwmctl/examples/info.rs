use libwmctl::prelude::*;
use prettytable::{format, Cell, Row, Table};

fn main() {
    let wm = info().unwrap();
    let win = active();

    println!("X11 Information");
    println!("-----------------------------------------------------------------------");
    println!("Window Manager: {}", wm.name);
    println!("Compositing:    {}", wm.compositing);
    println!("Root Window:    {}", wm.root_win_id);
    println!("Work area:      {}x{}", wm.work_area.0, wm.work_area.1);
    println!("Screen Size:    {}x{}", wm.screen_size.0, wm.screen_size.1);
    println!("Desktops:       {}", wm.desktops);
    println!("Active Window:  {}", win.id);
    println!();

    println!("Window Manager Supported Functions:");
    let mut table = Table::new();
    table.set_format(
        format::FormatBuilder::new()
            .separator(format::LinePosition::Top, format::LineSeparator::new('-', '+', '+', '+'))
            .separator(format::LinePosition::Title, format::LineSeparator::new('=', '+', '+', '+'))
            .padding(1, 1)
            .build(),
    );
    table.set_titles(Row::new(vec![Cell::new("NAME"), Cell::new("ID")]));

    let mut atoms = wm.supported.iter().collect::<Vec<_>>();
    atoms.sort_by(|a, b| a.1.cmp(b.1));
    for atom in atoms.iter() {
        table.add_row(Row::new(vec![Cell::new(&atom.1), Cell::new(&atom.0.to_string())]));
    }
    table.printstd();
}
