use clap::ArgMatches;
use libwmctl::prelude::*;
use witcher::prelude::*;

use crate::utils;

/// Run the info subcommand
///
/// ### Arguments
/// * `global` - the ArgMatches object for the global arguments
pub fn run(global: &ArgMatches) -> Result<()> {
    let id = utils::get_window_id(global, true);

    if let Some(matches) = global.subcommand_matches("move") {
        let pos = Position::try_from(matches.get_one::<String>("POSITION").unwrap().as_str()).pass()?;
        window(id).pos(pos).place().pass()?;

    // place
    } else if let Some(matches) = global.subcommand_matches("place") {
        let shape = Shape::try_from(matches.get_one::<String>("SHAPE").unwrap().as_str()).pass()?;
        let pos = Position::try_from(matches.get_one::<String>("POSITION").unwrap().as_str()).pass()?;
        window(id).shape(shape).pos(pos).place().pass()?;

    // static
    } else if let Some(matches) = global.subcommand_matches("static") {
        let w = matches.get_one::<String>("WIDTH").unwrap().parse::<u32>().pass()?;
        let h = matches.get_one::<String>("HEIGHT").unwrap().parse::<u32>().pass()?;
        let mut win = window(id).shape(Shape::Static(w, h));
        if let (Some(x), Some(y)) = (matches.get_one::<String>("X"), matches.get_one::<String>("Y")) {
            let x = x.parse::<i32>().pass()?;
            let y = y.parse::<i32>().pass()?;
            win = win.pos(Position::Static(x, y));
        }
        win.place().pass()?;

    // shape
    } else if let Some(matches) = global.subcommand_matches("shape") {
        let shape = Shape::try_from(matches.get_one::<String>("SHAPE").unwrap().as_str()).pass()?;
        window(id).shape(shape).place().pass()?;
    }

    Ok(())
}
