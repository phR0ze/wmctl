mod logger;
use std::env;
use gory::*;
use witcher::prelude::*;
use libwmctl::prelude::*;
use std::convert::TryFrom;
use clap::{App, AppSettings, Arg, SubCommand};
use tracing::Level;

pub const APP_NAME: &str = "wmctl";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const APP_GIT_COMMIT: &str = env!("APP_GIT_COMMIT");
pub const APP_BUILD_DATE: &str = env!("APP_BUILD_DATE");

fn init() -> Result<()> {

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

        // Version command
        .subcommand(SubCommand::with_name("version").alias("v").alias("ver").about("Print version information"))

        // Info
        .subcommand(SubCommand::with_name("info").about("List out X11 information")
            .long_about(r"List out X11 information i.e. resolution, workspace size, windows

Examples:

# List out X11 information
winctl info
"))
 
        // List out all the windows
        .subcommand(SubCommand::with_name("list").about("List out windows")
            .long_about(r"List out windows

Examples:

# List out windows
winctl list

# List out all X windows
winctl list -a
")
        .arg(Arg::with_name("all").short("a").long("all").takes_value(false).help("Show all X windows not just WM windows"))
        )

        // Move window to given position
        .subcommand(SubCommand::with_name("move").about("Move the active window")
            .long_about(r"Move the active window

Examples:

# Move the active window to the center
winctl move center

# Move the active window to the right edge of the screen
winctl move right

# Move the active window to the bottom center of the screen
winctl move bottom-center
")
            .arg(Arg::with_name("POSITION").index(1).required(true)
                .value_names(&["center", "left", "right", "top", "bottom", "top-left", "top-right", "bottom-right", "bottom-left", "left-center", "right-center", "top-center", "bottom-center"])
                .help("position to move the active window to"))
        )

        // Resize and center
        .subcommand(SubCommand::with_name("resize").about("Resize and center the active window")
            .long_about(r"Resize and center the active window

Examples:

# w and h are int values 1-100 treated as a percentage of the total screen size
winctl resize 70 80
")
            .arg(Arg::with_name("W_RATIO").index(1).required(true).help("w ratio of total display size to use (1 - 100)"))
            .arg(Arg::with_name("H_RATIO").index(2).required(true).help("h ratio of total display size to use (1 - 100)")),
        )

        // Shape
        .subcommand(SubCommand::with_name("shape").about("Shape the active window")
            .long_about(r"Shape the active window

Examples:

# Grow the active window by 10% on all sides
winctl shape grow

# Shrink the active window by 10% on all sides
winctl shape shrink

# Shape the active window as 4x3 ratio
winctl shape 4x3

# Shape the active window to be large i.e. 90% of the current screen size
winctl shape large
")
            .arg(Arg::with_name("SHAPE").index(1).required(true)
                .value_names(&["4x3", "small", "large", "grow", "max", "shrink", "unmax"])
                .help("shape directive to use against the active window"))
        )
        .get_matches_from_safe(env::args_os()).pass()?;

    // Execute
    // ---------------------------------------------------------------------------------------------
    logger::init(match matches.is_present("debug") {
        true => Some(Level::DEBUG),
        _ => None,
    });

    // Determine the target window
    let win = {
        matches.value_of("window").and_then(|x| x.parse::<u32>().ok())
    };

    // Version
    if let Some(ref _matches) = matches.subcommand_matches("version") {
        println!("{}: {}", APP_NAME.cyan(), APP_DESCRIPTION.cyan());
        println!("{}", "--------------------------------------------------------".cyan());
        println!("{:<w$} {}", "Version:", APP_VERSION, w = 18);
        println!("{:<w$} {}", "Build Date:", APP_BUILD_DATE, w = 18);
        println!("{:<w$} {}", "Git Commit:", APP_GIT_COMMIT, w = 18);

    // info
    } else if let Some(_) = matches.subcommand_matches("info") {
        libwmctl::info(win).pass()?;

    // list
    } else if let Some(matches) = matches.subcommand_matches("list") {
        libwmctl::list(matches.is_present("all")).pass()?;

    // move
    } else if let Some(ref matches) = matches.subcommand_matches("move") {
        let position = WinPosition::try_from(matches.value_of("POSITION").unwrap()).pass()?;
        libwmctl::move_win(win, position).pass()?;

    // resize
    } else if let Some(ref matches) = matches.subcommand_matches("resize") {
        let x_ratio = matches.value_of("W_RATIO").unwrap().parse::<u32>().wrap("Failed to convert W_RATIO into a valid 1-100 int")?;
        let y_ratio = matches.value_of("H_RATIO").unwrap().parse::<u32>().wrap("Failed to convert Y_RATIO into a valid 1-100 int")?;
        libwmctl::resize_and_center(x_ratio, y_ratio).pass()?;

    // shape
    } else if let Some(ref matches) = matches.subcommand_matches("shape") {
        let shape = WinShape::try_from(matches.value_of("SHAPE").unwrap()).pass()?;
        libwmctl::shape_win(win, shape).pass()?;
    }

    Ok(())
}

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