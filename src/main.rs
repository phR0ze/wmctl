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

use clap::{App, AppSettings, Arg, SubCommand};
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
    let matches = App::new(format!("{}", APP_NAME.cyan()))
        .version(&format!("v{}", APP_VERSION)[..])
        .about(&format!("{}", APP_DESCRIPTION.green())[..])
        .setting(AppSettings::SubcommandRequiredElseHelp)

        // Global flags
        .arg(Arg::with_name("test").short("t").long("test").takes_value(false).help("Enable test mode"))
        .arg(Arg::with_name("debug").short("d").long("debug").takes_value(false).help("Enable debug logging"))
        .arg(Arg::with_name("quiet").short("q").long("quiet").takes_value(false).help("Disable all logging"))

        // Global options
        .arg(Arg::with_name("loglevel").long("log-level").value_name("NAME").takes_value(true).help("Sets the log level [error|warn|info|debug|trace] [default: info]"))
        .arg(Arg::with_name("window").short("w").long("window").value_name("WINDOW").takes_value(true).help("Window to operate against"))
        .arg(Arg::with_name("class").short("c").long("class").value_name("CLASS").takes_value(true).help("Class of window to operate against (first matching)"))

        // Version command
        .subcommand(SubCommand::with_name("version").alias("v").alias("ver").about("Print version information"))

        // Info
        .subcommand(SubCommand::with_name("info").about("Print X11 component information")
            .long_about(r"Print out X11 component information e.g. Window Manager, Window or other

Examples:

# Print out the active window information
wmctl info

# Print out information for the first window by class
wmctl -c firefox info

# Print out Window Manager information
wmctl info winmgr
").subcommand(SubCommand::with_name("winmgr").about("Print out information for the Window Manager")
    .arg(Arg::with_name("all").long("all").short("a").takes_value(false).help("Show supported Window Manager functions"))))

        // List out all the windows
        .subcommand(SubCommand::with_name("list").about("List out windows")
            .long_about(r"List out windows

Examples:

# List out windows
wmctl list

# List out all X windows
wmctl list -a
")
        .arg(Arg::with_name("all").short("a").long("all").takes_value(false).help("Show all X windows not just WM windows"))
        )

        // Move
        .subcommand(SubCommand::with_name("move").about("Move the active window")
            .long_about(r"Move the active window

Examples:

# Move the active window to the center
wmctl move center

# Move the active window to the right edge of the screen
wmctl move right

# Move the active window to the bottom center of the screen
wmctl move bottom-center
")
            .arg(Arg::with_name("POSITION").index(1).required(true)
                .value_names(&["center", "left", "right", "top", "bottom", "top-left", "top-right", "bottom-right", "bottom-left", "left-center", "right-center", "top-center", "bottom-center"])
                .help("position to move the active window to"))
        )

        // Place
        .subcommand(SubCommand::with_name("place").about("Shape and move the window")
            .long_about(r"Shape and move the window

Examples:

# Shape the active window to half the width but full height and position to the right
wmctl place halfw right

# Shape the active window to be small and position bottom left
wmctl place small bottom-left
")
            .arg(Arg::with_name("SHAPE").index(1).required(true)
                .value_names(&["halfh", "halfw", "small", "medium", "large", "grow", "max", "shrink", "unmax"])
                .help("shape directive to use against the window"))
            .arg(Arg::with_name("POSITION").index(2).required(true)
                .value_names(&["center", "left", "right", "top", "bottom", "top-left", "top-right", "bottom-right", "bottom-left", "left-center", "right-center", "top-center", "bottom-center"])
                .help("position to move the window to"))
        )

        // Shape
        .subcommand(SubCommand::with_name("shape").about("Shape the window")
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
            .arg(Arg::with_name("SHAPE").index(1).required(true)
                .value_names(&["halfh", "halfw", "small", "medium", "large", "grow", "max", "shrink", "unmax"])
                .help("shape directive to use against the window"))
        )

        // Static
        .subcommand(SubCommand::with_name("static").about("Resize and move the window")
            .long_about(r"Resize and move the window statically

Examples:

# w and h are static values of the size of the window
wmctl static 1276 757

# w and h are static values of the size of the window and x, y are the intended location
wmctl static 1276 757 0 0
")
            .arg(Arg::with_name("WIDTH").index(1).required(true).help("width of the window"))
            .arg(Arg::with_name("HEIGHT").index(2).required(true).help("height of the window"))
            .arg(Arg::with_name("X").index(3).required(false).help("x location of the window"))
            .arg(Arg::with_name("Y").index(4).required(false).help("y location of the window"))
        )
        .get_matches_from_safe(env::args_os()).pass()?;

    // Execute
    // ---------------------------------------------------------------------------------------------
    init_logging(match matches.is_present("debug") {
        true => Some(Level::DEBUG),
        _ => None,
    });

    // Version
    if let Some(ref _matches) = matches.subcommand_matches("version") {
        println!("{}: {}", APP_NAME.cyan(), APP_DESCRIPTION.cyan());
        println!("{}", "--------------------------------------------------------".cyan());
        println!("{:<w$} {}", "Version:", APP_VERSION, w = 18);
        println!("{:<w$} {}", "Build Date:", APP_BUILD_DATE, w = 18);
        println!("{:<w$} {}", "Git Commit:", APP_GIT_COMMIT, w = 18);

    // info
    } else if matches.is_present("info") {
        info::run(&matches);

    // list
    } else if matches.is_present("list") {
        list::run(&matches)?;

    // place
    } else if matches.is_present("move")
        || matches.is_present("place")
        || matches.is_present("shape")
        || matches.is_present("static")
    {
        place::run(&matches)?;
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
