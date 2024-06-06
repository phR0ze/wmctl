use libwmctl::prelude::*;
use prettytable::{format, Cell, Row, Table};

// Get all window properties for the first window that matches the given class
fn main() {
    let mut table = Table::new();
    table.set_format(
        format::FormatBuilder::new()
            .separator(format::LinePosition::Top, format::LineSeparator::new('-', '+', '+', '+'))
            .separator(format::LinePosition::Title, format::LineSeparator::new('=', '+', '+', '+'))
            .padding(1, 1)
            .build(),
    );
    table.set_titles(Row::new(vec![Cell::new("NAME"), Cell::new("ID"), Cell::new("VALUE")]));

    println!("==============================================");
    let win = first_by_class("firefox").unwrap();
    println!("Properties for class={}, id={}", win.class().unwrap(), win.id);
    let props = win.properties().unwrap();
    for prop in props.iter() {
        table.add_row(Row::new(vec![Cell::new(&prop.name), Cell::new(&prop.id.to_string())]));
    }
    table.printstd();
}
