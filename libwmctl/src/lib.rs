use std::ops::Deref;

//use tracing::{info};
use xcb;
use xcb_util::ewmh;

/// `Result<T>` provides a simplified result type with a common error type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// All essential symbols in a simple consumable form
///
/// ### Examples
/// ```
/// use libwmctl::prelude::*;
/// ```
pub mod prelude {
    pub use crate::*;
}

struct Display {
    conn: ewmh::Connection,
    screen: i32,
    width: i32,
    height: i32,
}
impl Deref for Display {
	type Target = xcb::Connection;

	fn deref(&self) -> &Self::Target {
		&self.conn
	}
}

/// Resize the active window based on the ratio of the overall screen size then center it
pub fn resize_and_center(x_ratio: f64, y_ratio: f64) -> Result<()> {
    let display = init()?;
    let win = active_window(&display)?;

    // Remove maximizing states
    ewmh::request_change_wm_state(&display.conn, display.screen, win, ewmh::STATE_REMOVE, display.conn.WM_ACTION_MAXIMIZE_HORZ(), display.conn.WM_STATE_MAXIMIZED_VERT(), 0).request_check()?;

    // Calculate window size
    let (w, h) =  ((display.width as f64 * x_ratio) as i32, (display.height as f64 * y_ratio) as i32);

    // Center the window on the screen
    let status_bar = 26;
    let (x, y) =  ((display.width - w)/2, (display.height - h - status_bar)/2);

    // Resize and position
    let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y | ewmh::MOVE_RESIZE_WINDOW_WIDTH | ewmh::MOVE_RESIZE_WINDOW_HEIGHT;
    ewmh::request_move_resize_window(&display.conn, display.screen, win, 0, 0, flags, x as u32, y as u32, w as u32, h as u32).request_check()?;
    display.flush();
    Ok(())
}

/// List out all the current window ids and their titles
pub fn list_windows() -> Result<()> {
    let display = init()?;
    for win_id in ewmh::get_client_list(&display.conn, display.screen).get_reply()?.windows() {

        //  Some window values don't appear to be valid and need to be skipped
        if let Ok(name) = win_title(&display, *win_id) {
            println!("ID: {}, Name: {}", *win_id, name);
        }
    }
    Ok(())
}

// Connect to the X11 server
fn init() -> Result<Display> {
    let (conn, screen_id) = xcb::Connection::connect(None)?;
    let (width, height) = {
        let screen = conn.get_setup().roots().nth(screen_id as usize).unwrap();
        (screen.width_in_pixels(), screen.height_in_pixels())
    };
    Ok(Display{
        conn: ewmh::Connection::connect(conn).map_err(|(e, _)| e)?,
        screen: screen_id, width: width as i32, height: height as i32
    })
}

// Get window title
fn win_title(display: &Display, win: xcb::Window) -> Result<String> {
    let name = ewmh::get_wm_name(&display.conn, win).get_reply()?;
    Ok(name.string().to_string())
}

/// Get the active window id
fn active_window(display: &Display) -> Result<u32> {
    let active_win = ewmh::get_active_window(&display.conn, display.screen).get_reply()?;
    Ok(active_win)
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
