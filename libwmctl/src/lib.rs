mod error;
mod position;
use error::*;
use position::*;
use std::{str, ops::Deref};

//use tracing::{info};
use xcb;
use xcb_util::ewmh;

/// All essential symbols in a simple consumable form
///
/// ### Examples
/// ```
/// use libwmctl::prelude::*;
/// ```
pub mod prelude {
    pub use crate::*;
    pub use position::*;
}

struct Display {
    conn: ewmh::Connection,     // window manager connection
    screen: i32,                // screen number
    full_width: i32,            // screen width
    full_height: i32,           // screen height
    work_width: i32,            // screen width minus possible taskbar
    work_height: i32,           // screen height minus possible taskbar
}
impl Deref for Display {
	type Target = xcb::Connection;

	fn deref(&self) -> &Self::Target {
		&self.conn
	}
}

/// Move the active window without changing its size
pub fn move_win(position: Position) -> WmCtlResult<()> {
    let display = init()?;
    let win = active_window(&display)?;
    remove_maximize(&display, win)?;

    // Value returned for y is 28 off??
    let (x, y, w, h) = win_geometry(&display, win)?;
    println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);

    // right
    let x = display.work_width - w;
    let y = 0; // seems to be 28 off by default?
    //let y = display.work_height - h;
    println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);

    println!("w: {}, h: {}", display.full_width, display.full_height);
    println!("w: {}, h: {}", display.work_width, display.work_height);

    // Resize and position
    let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y | ewmh::MOVE_RESIZE_WINDOW_WIDTH | ewmh::MOVE_RESIZE_WINDOW_HEIGHT;
    ewmh::request_move_resize_window(&display.conn, display.screen, win, 0, 0, flags, x as u32, y as u32, w as u32, h as u32).request_check()?;
    display.flush();
    Ok(())
}

/// Resize the active window based on the ratio of the overall screen size then center it
pub fn resize_and_center(x_ratio: f64, y_ratio: f64) -> WmCtlResult<()> {
    let display = init()?;
    let win = active_window(&display)?;

    // Remove maximizing states
    remove_maximize(&display, win)?;

    // Calculate window size
    let (w, h) =  ((display.work_width as f64 * x_ratio) as i32, (display.work_height as f64 * y_ratio) as i32);

    // Center the window on the screen
    let (x, y) =  ((display.work_width - w)/2, (display.work_height - h)/2);

    // Resize and position
    let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y | ewmh::MOVE_RESIZE_WINDOW_WIDTH | ewmh::MOVE_RESIZE_WINDOW_HEIGHT;
    ewmh::request_move_resize_window(&display.conn, display.screen, win, 0, 0, flags, x as u32, y as u32, w as u32, h as u32).request_check()?;
    display.flush();
    Ok(())
}

/// List out all the current window ids and their titles
pub fn list_windows() -> WmCtlResult<()> {
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
fn init() -> WmCtlResult<Display> {
    let (conn, screen) = xcb::Connection::connect(None)?;

    // Get the full screen size
    let (width, height) = {
        let screen = conn.get_setup().roots().nth(screen as usize).unwrap();
        (screen.width_in_pixels(), screen.height_in_pixels())
    };

    // Get the adjusted workspace size i.e. screen full size minus taskbar
    let conn = ewmh::Connection::connect(conn).map_err(|(e, _)| e)?;
    let (work_width, work_height) = {
        let reply = ewmh::get_work_area(&conn, screen).get_reply()?;
        let area = reply.work_area().first().unwrap();
        (area.width(), area.height())
    };
    Ok(Display{
        conn: conn,
        screen: screen,
        full_width: width as i32,
        full_height: height as i32, 
        work_width: work_width as i32,
        work_height: work_height as i32,
    })
}

// Remove maximizing attributes
fn remove_maximize(display: &Display, win: xcb::Window) -> WmCtlResult<()> {
    ewmh::request_change_wm_state(&display.conn, display.screen, win, ewmh::STATE_REMOVE,
        display.conn.WM_ACTION_MAXIMIZE_HORZ(), display.conn.WM_STATE_MAXIMIZED_VERT(), 0).request_check()?;
    Ok(())
}


// Get desktop work area
fn work_area(display: &Display, win: xcb::Window) -> WmCtlResult<(i16, i16, u16, u16)> {
    let geo = xcb::get_geometry(&display, win).get_reply()?;
    Ok((geo.x(), geo.y(), geo.width(), geo.height()))
}

// Get window geometry
fn win_geometry(display: &Display, win: xcb::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
    let geo = xcb::get_geometry(&display, win).get_reply()?;
    Ok((geo.x() as i32, geo.y() as i32, geo.width() as i32, geo.height() as i32))
}

// Get window pid
fn win_pid(display: &Display, win: xcb::Window) -> WmCtlResult<u32> {
    let pid = ewmh::get_wm_pid(&display.conn, win).get_reply()?;
    Ok(pid)
}

// Get window title
fn win_title(display: &Display, win: xcb::Window) -> WmCtlResult<String> {
    let name = ewmh::get_wm_name(&display.conn, win).get_reply()?;
    Ok(name.string().to_string())
}

/// Get the active window id
fn active_window(display: &Display) -> WmCtlResult<u32> {
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
