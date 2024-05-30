use libwmctl::prelude::*;
use prettytable::{format, Cell, Row, Table};
use witcher::prelude::*;

pub fn first_by_class(class: String) -> Result<()> {
    let windows = libwmctl::windows(false).pass()?;

    Ok(())
}
