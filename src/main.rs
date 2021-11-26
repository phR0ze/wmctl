mod logger;
use gory::*;
use libwmctl;

pub const APP_NAME: &str = "wmctl";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const APP_GIT_COMMIT: &str = env!("APP_GIT_COMMIT");
pub const APP_BUILD_DATE: &str = env!("APP_BUILD_DATE");

fn main() {
    logger::init();

    println!("\n{} - {}", APP_NAME.cyan(), APP_DESCRIPTION);
    println!("{:->w$}", "-", w = 60);
    println!("{:<w$} {}", "Version:", APP_VERSION, w = 18);
    println!("{:<w$} {}", "Build Date:", APP_BUILD_DATE, w = 18);
    println!("{:<w$} {}", "Git Commit:", APP_GIT_COMMIT, w = 18);

    libwmctl::info();
}

// fn info() {

// }