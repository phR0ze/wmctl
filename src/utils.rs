use clap::ArgMatches;

/// Log the message and exit
///
/// ### Arguments
/// * `msg` - the message to log
fn fatal(msg: &str) {
    println!("{}", msg);
    std::process::exit(1);
}

/// Get the window id from the various methods
///
/// ### Arguments
/// * `matches` - the ArgMatches object to search
/// * `active` - if true, get the active window if no other method is given
pub fn get_window_id(matches: &ArgMatches, active: bool) -> u32 {
    let mut id = if let Some(window) = matches.get_one::<String>("window") {
        let id = window.parse::<u32>().ok();
        if id.is_none() {
            fatal(&format!("Invalid Window identifier: {}", window));
        }
        id
    } else if let Some(class) = matches.get_one::<String>("class") {
        let id = libwmctl::first_by_class(class).map(|x| x.id);
        if id.is_none() {
            fatal(&format!("Not found Window class: {}", class));
        }
        id
    } else {
        None
    };

    // Use the active window if no other method is given and authorized
    if id.is_none() {
        if active {
            id = Some(libwmctl::active().id);
        } else {
            fatal("Window identifier was not given");
        }
    }
    id.unwrap()
}
