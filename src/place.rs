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
        let pos = Position::try_from(matches.value_of("POSITION").unwrap()).pass()?;
        window(id).pos(pos).place().pass()?;

    // place
    } else if let Some(matches) = global.subcommand_matches("place") {
        let shape = Shape::try_from(matches.value_of("SHAPE").unwrap()).pass()?;
        let pos = Position::try_from(matches.value_of("POSITION").unwrap()).pass()?;
        window(id).shape(shape).pos(pos).place().pass()?;

    // static
    } else if let Some(matches) = global.subcommand_matches("static") {
        let w = matches.value_of("WIDTH").unwrap().parse::<u32>().pass()?;
        let h = matches.value_of("HEIGHT").unwrap().parse::<u32>().pass()?;
        let mut win = window(id).shape(Shape::Static(w, h));
        if matches.value_of("X").is_some() && matches.value_of("Y").is_some() {
            let x = matches.value_of("X").unwrap().parse::<i32>().pass()?;
            let y = matches.value_of("Y").unwrap().parse::<i32>().pass()?;
            win = win.pos(Position::Static(x, y));
        }
        win.place().pass()?;

    // shape
    } else if let Some(matches) = global.subcommand_matches("shape") {
        let shape = Shape::try_from(matches.value_of("SHAPE").unwrap()).pass()?;
        window(id).shape(shape).place().pass()?;
    }

    Ok(())
}
