mod logger;
use std::env;
use gory::*;
use libwmctl;
use clap::{App, AppSettings, Arg, SubCommand};

pub const APP_NAME: &str = "wmctl";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const APP_GIT_COMMIT: &str = env!("APP_GIT_COMMIT");
pub const APP_BUILD_DATE: &str = env!("APP_BUILD_DATE");

fn main() {

    // Parse cli args
    // -----------------------------------------------------------------------------------------
    let matches = App::new(format!("{}", APP_NAME.cyan()))
        .version(&format!("v{}", APP_VERSION)[..])
        .about(&format!("{}", APP_DESCRIPTION.green())[..])
        .setting(AppSettings::SubcommandRequiredElseHelp)

        // Global arguments
        .arg(Arg::with_name("test").short("t").long("test").takes_value(false).help("Enable test mode"))
        .arg(Arg::with_name("debug").short("d").long("debug").takes_value(false).help("Enable debug logging"))
        .arg(Arg::with_name("quiet").short("q").long("quiet").takes_value(false).help("Disable all logging"))
        .arg(Arg::with_name("loglevel").long("log-level").value_name("NAME").takes_value(true).help("Sets the log level [error|warn|info|debug|trace] [default: info]"),
        )

        // Version command
        .subcommand(SubCommand::with_name("version").alias("v").alias("ver").about("Print version information"))

        // Resize and center
        .subcommand(
            SubCommand::with_name("resize")
                .about("Resize and center the active window")
                .long_about(
                    r"Resize and center the active window

Examples:

winctl resize 0.70 0.80
",)
                .arg(Arg::with_name("X_RATIO").index(1).required(true).help("x ratio of total display size to use"))
                .arg(Arg::with_name("Y_RATIO").index(2).required(true).help("y ratio of total display size to use")),
        )
        .get_matches_from_safe(env::args_os());

    // Initialize winctl
    // ---------------------------------------------------------------------------------------------
    logger::init();

    // Execute
    // ---------------------------------------------------------------------------------------------
    if let Ok(matches) = matches {

        // Version
        if let Some(ref _matches) = matches.subcommand_matches("version") {
            println!("{}: {}", APP_NAME.cyan(), APP_DESCRIPTION.cyan());
            println!("{}", "--------------------------------------------------------".cyan());
            println!("{:<w$} {}", "Version:", APP_VERSION, w = 18);
            println!("{:<w$} {}", "Build Date:", APP_BUILD_DATE, w = 18);
            println!("{:<w$} {}", "Git Commit:", APP_GIT_COMMIT, w = 18);

        // Resize
        } else if let Some(ref matches) = matches.subcommand_matches("resize") {
            //let components: Vec<&str> = matches.values_of("components").unwrap().collect();
            //println!("{:?}", components)
            //libwmctl::resize_and_center(0.70, 0.80)?;
        }
    } else {
        println!("{}", matches.unwrap_err());
        // match matches.unwrap_err().downcast_ref::<clap::Error>() {
        //     Some(clap) => println!("{}", clap),
        //     None => println!("{:?}", err),
        // };
    }
}