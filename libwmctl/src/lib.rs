pub mod model;

use std::str;
use x11rb::connection::Connection;
use x11rb::rust_connection::RustConnection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};
use tracing::{debug};

/// `Result<T>` provides a simplified result type with a common error type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// All essential symbols in a simple consumable form
///
/// ### Examples
/// ```
/// use libwmctl::prelude::*;
/// ```
pub mod prelude {
    pub use crate::model;
}

/// Open a connectdion to the X11 server
pub fn connect() -> Result<(RustConnection, model::Display)> {

    // TODO: handle more than one screen
    debug!("Connecting to the X server");
    let (conn, number) = x11rb::connect(None)?;

    // Extract display information
    let screen = &conn.setup().roots[number];
    conn.flush()?;
 
    let display = model::Display {
        width: screen.width_in_pixels,
        height: screen.height_in_pixels,
        number: number,
    };

    return Ok((conn, display))
}

pub fn test() -> Result<()> {

    // Get the current screen
    let (conn, number) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[number];

    // Query for all existing windows
    let tree = conn.query_tree(screen.root)?.reply()?;
    for win in tree.children {
        let name = get_win_name(&conn, win)?;
        if name != "" {
            println!("Win: {}", name);
        }
    }

    conn.flush()?;
    Ok(())
}

// _NET_ACTIVE_WINDOW
// https://docs.rs/cnx/0.3.0/cnx/widgets/struct.ActiveWindowTitle.html
// https://github.com/meh/rust-xcb-util/blob/master/src/ewmh.rs
// https://crates.io/crates/cnx
// https://crates.io/crates/xcb-util

// Get the given window's name
pub fn get_win_name(conn: &RustConnection, win: u32) -> Result<String> {
    let atom = conn.intern_atom(true, b"_NET_WM_NAME")?.reply()?.atom;
    let res = conn.get_property(false, win, atom, AtomEnum::STRING, 0, std::u32::MAX)?.reply()?;
    //let res = conn.get_property(false, win, AtomEnum::WM_NAME, AtomEnum::STRING, 0, std::u32::MAX)?.reply()?;
    Ok(str::from_utf8(&res.value)?.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
