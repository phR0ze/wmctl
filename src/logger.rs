use std::env;
use tracing::Level;
use tracing_subscriber;

// Configure logging
pub fn init(level: Option<Level>) {

    // Use the given log level as highest priority
    // Use environment log level as second priority
    // Fallback on INFO if neither is set
    let loglevel = match level {
        Some(x) => x,
        None => match env::var("LOG_LEVEL") {
            Ok(val) => val.parse().unwrap_or(Level::INFO),
            Err(_e) => Level::INFO, // default to Info
        }
    };
    tracing_subscriber::fmt()
        .with_target(false) // turn off file name
        .with_max_level(loglevel) // set max level to log
        //.json() // uncomment this line to convert it into json output
        .init();
}