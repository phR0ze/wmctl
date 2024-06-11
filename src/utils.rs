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
    let mut id = if matches.is_present("window") {
        let id = matches.value_of("window").unwrap().parse::<u32>().ok();
        if id.is_none() {
            fatal(&format!("Invalid Window identifier: {}", matches.value_of("window").unwrap()));
        }
        id
    } else if matches.is_present("class") {
        let id = matches.value_of("class").and_then(|x| libwmctl::first_by_class(x).and_then(|x| Some(x.id)));
        if id.is_none() {
            fatal(&format!("Not found Window class: {}", matches.value_of("class").unwrap()));
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
