//! `wmctl` implements a subset of the [Extended Window Manager Hints (EWMH)
//! specification](https://specifications.freedesktop.org/wm-spec/latest/) as a way to work along
//! side EWMH compatible window managers as a companion. `wmctl` provides the ability to precisely
//! define how windows should be shaped and placed and can fill in gaps for window managers lacking
//! some shaping or placement features. Mapping `wmctl` commands to user defined hot key sequences
//! will allow for easy window manipulation beyond what your favorite EWMH window manager provides.
//!
//! ## Command line examples
//!
//! ### Shape a window
//! Shape the active window using the pre-defined `small` shape which is a quarter of the screen.
//! ```bash
//! wmctl shape small
//! ```
//!
//! ### Move a window
//! Move the active window to the bottom left corner of the screen.
//! ```bash
//! wmctl move bottom-left
//! ```
//!
//! ### Place a window
//! Shape the active window using the pre-defined `small` shape which is a quarter of the screen
//! and then position it in the bottom left corner of the screen.
//! ```bash
//! wmctl place small bottom-left
//! ```
use std::env;

use clap::{Arg, ArgAction, Command};
use gory::*;
use tracing::Level;
use tracing_subscriber;
use witcher::prelude::*;

mod info;
mod list;
mod place;
mod utils;

// Configure logging
#[doc(hidden)]
fn init_logging(level: Option<Level>) {
    // Use the given log level as highest priority
    // Use environment log level as second priority
    // Fallback on INFO if neither is set
    let loglevel = match level {
        Some(x) => x,
        None => match env::var("LOG_LEVEL") {
            Ok(val) => val.parse().unwrap_or(Level::INFO),
            Err(_e) => Level::INFO, // default to Info
        },
    };
    tracing_subscriber::fmt()
        .with_target(false) // turn off file name
        .with_max_level(loglevel) // set max level to log
        //.json() // uncomment this line to convert it into json output
        .init();
}

#[doc(hidden)]
fn init() -> Result<()> {
    const APP_NAME: &str = "wmctl";
    const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
    const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
    const APP_GIT_COMMIT: &str = env!("APP_GIT_COMMIT");
    const APP_BUILD_DATE: &str = env!("APP_BUILD_DATE");

    // Parse cli args
    // -----------------------------------------------------------------------------------------
    let matches = Command::new(format!("{}", APP_NAME.cyan()))
        .version(format!("v{}", APP_VERSION))
        .about(format!("{}", APP_DESCRIPTION.green()))
        .subcommand_required(true)
        .arg_required_else_help(true)

        // Global flags
        .arg(Arg::new("test").short('t').long("test").action(ArgAction::SetTrue).help("Enable test mode"))
        .arg(Arg::new("debug").short('d').long("debug").action(ArgAction::SetTrue).help("Enable debug logging"))
        .arg(Arg::new("quiet").short('q').long("quiet").action(ArgAction::SetTrue).help("Disable all logging"))

        // Global options
        .arg(Arg::new("loglevel").long("log-level").value_name("NAME").help("Sets the log level [error|warn|info|debug|trace] [default: info]"))
        .arg(Arg::new("window").short('w').long("window").value_name("WINDOW").help("Window to operate against"))
        .arg(Arg::new("class").short('c').long("class").value_name("CLASS").help("Class of window to operate against (first matching)"))

        // Version command
        .subcommand(Command::new("version").alias("v").alias("ver").about("Print version information"))

        // Info
        .subcommand(Command::new("info").about("Print X11 component information")
            .long_about(r"Print out X11 component information e.g. Window Manager, Window or other

Examples:

# Print out the active window information
wmctl info

# Print out information for the first window by class
wmctl -c firefox info

# Print out Window Manager information
wmctl info winmgr
").subcommand(Command::new("winmgr").about("Print out information for the Window Manager")
    .arg(Arg::new("all").long("all").short('a').action(ArgAction::SetTrue).help("Show supported Window Manager functions"))))

        // List out all the windows
        .subcommand(Command::new("list").about("List out windows")
            .long_about(r"List out windows

Examples:

# List out windows
wmctl list

# List out all X windows
wmctl list -a
")
        .arg(Arg::new("all").short('a').long("all").action(ArgAction::SetTrue).help("Show all X windows not just WM windows"))
        )

        // Move
        .subcommand(Command::new("move").about("Move the active window")
            .long_about(r"Move the active window

Examples:

# Move the active window to the center
wmctl move center

# Move the active window to the right edge of the screen
wmctl move right

# Move the active window to the bottom center of the screen
wmctl move bottom-center
")
            .arg(Arg::new("POSITION").index(1).required(true)
                .value_parser(["center", "left", "right", "top", "bottom", "top-left", "top-right", "bottom-right", "bottom-left", "left-center", "right-center", "top-center", "bottom-center"])
                .help("position to move the active window to"))
        )

        // Place
        .subcommand(Command::new("place").about("Shape and move the window")
            .long_about(r"Shape and move the window

Examples:

# Shape the active window to half the width but full height and position to the right
wmctl place halfw right

# Shape the active window to be small and position bottom left
wmctl place small bottom-left
")
            .arg(Arg::new("SHAPE").index(1).required(true)
                .value_parser(["halfh", "halfw", "small", "medium", "large", "grow", "max", "shrink", "unmax"])
                .help("shape directive to use against the window"))
            .arg(Arg::new("POSITION").index(2).required(true)
                .value_parser(["center", "left", "right", "top", "bottom", "top-left", "top-right", "bottom-right", "bottom-left", "left-center", "right-center", "top-center", "bottom-center"])
                .help("position to move the window to"))
        )

        // Shape
        .subcommand(Command::new("shape").about("Shape the window")
            .long_about(r"Shape the window

Examples:

# Grow the active window by 10% on all sides
wmctl shape grow

# Shrink the active window by 10% on all sides
wmctl shape shrink

# Shape the active window to be large i.e. 4x3 ~50% of the current screen size
wmctl shape medium

# Shape the active window to be large i.e. 4x3 ~90% of the current screen size
wmctl shape large
")
            .arg(Arg::new("SHAPE").index(1).required(true)
                .value_parser(["halfh", "halfw", "small", "medium", "large", "grow", "max", "shrink", "unmax"])
                .help("shape directive to use against the window"))
        )

        // Static
        .subcommand(Command::new("static").about("Resize and move the window")
            .long_about(r"Resize and move the window statically

Examples:

# w and h are static values of the size of the window
wmctl static 1276 757

# w and h are static values of the size of the window and x, y are the intended location
wmctl static 1276 757 0 0
")
            .arg(Arg::new("WIDTH").index(1).required(true).help("width of the window"))
            .arg(Arg::new("HEIGHT").index(2).required(true).help("height of the window"))
            .arg(Arg::new("X").index(3).required(false).help("x location of the window"))
            .arg(Arg::new("Y").index(4).required(false).help("y location of the window"))
        )
        .try_get_matches_from(env::args_os()).pass()?;

    // Execute
    // ---------------------------------------------------------------------------------------------
    init_logging(match matches.get_flag("debug") {
        true => Some(Level::DEBUG),
        _ => None,
    });

    // Version
    match matches.subcommand() {
        Some(("version", _)) => {
            println!("{}: {}", APP_NAME.cyan(), APP_DESCRIPTION.cyan());
            println!("{}", "--------------------------------------------------------".cyan());
            println!("{:<w$} {}", "Version:", APP_VERSION, w = 18);
            println!("{:<w$} {}", "Build Date:", APP_BUILD_DATE, w = 18);
            println!("{:<w$} {}", "Git Commit:", APP_GIT_COMMIT, w = 18);
        },

        // info
        Some(("info", _)) => info::run(&matches),

        // list
        Some(("list", _)) => list::run(&matches)?,

        // place
        Some(("move" | "place" | "shape" | "static", _)) => place::run(&matches)?,

        _ => {},
    }

    Ok(())
}

#[doc(hidden)]
fn main() {
    match init() {
        Ok(_) => 0,
        Err(err) => {
            match err.downcast_ref::<clap::Error>() {
                Some(clap) => println!("{}", clap),
                None => println!("{:?}", err),
            };
            1
        },
    };
}
