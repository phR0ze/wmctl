use crate::{WmCtlResult, prelude::Position};
use std::ops::Deref;
use xcb;
use xcb_util::ewmh;

pub(crate) struct WmCtl {
    pub(crate) conn: ewmh::Connection,  // window manager connection
    pub(crate) screen: i32,             // screen number
    pub(crate) full_width: i32,         // screen width
    pub(crate) full_height: i32,        // screen height
    pub(crate) work_width: i32,         // screen width minus possible taskbar
    pub(crate) work_height: i32,        // screen height minus possible taskbar
}

impl Deref for WmCtl {
	type Target = xcb::Connection;

	fn deref(&self) -> &Self::Target {
		&self.conn
	}
}

// Connect to the X11 server
impl WmCtl {
    pub(crate) fn open() -> WmCtlResult<WmCtl> {
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
        Ok(WmCtl{
            conn,
            screen,
            full_width: width as i32,
            full_height: height as i32, 
            work_width: work_width as i32,
            work_height: work_height as i32,
        })
    }

    // Move and resize window
    pub(crate) fn move_and_resize(&self, win: xcb::Window, x: i32, y: i32, w: i32, h: i32) -> WmCtlResult<()> {
        //let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y | ewmh::MOVE_RESIZE_WINDOW_WIDTH | ewmh::MOVE_RESIZE_WINDOW_HEIGHT;
        let (x, y, w, h) = (500, 00, 1000, 1000);
        let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y;
        ewmh::request_move_resize_window(&self.conn, self.screen, win, 0, 0, flags, x as u32, y as u32, w as u32, h as u32).request_check()?;
        self.flush();
        Ok(())
    }

    // Move window
    pub(crate) fn move_win(&self, win: xcb::Window, position: Position) -> WmCtlResult<()> {
        let (w, h) = (0, 0); // not used since flags don't indicate resize
        let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y;

        // Compute coordinates based on position
        let (x, y) = match position {
            Position::Center => (0, 0),
            Position::Left => (0, 0),
            Position::Right => (0, 0),
            Position::Top => (0, 0),
            Position::Bottom => (0, 0),
            Position::TopLeft => (0, 0),
            Position::TopRight => (0, 0),
            Position::BottomLeft => (0, 0),
            Position::BottomRight => (0, 0),
        };

        ewmh::request_move_resize_window(&self.conn, self.screen, win, 0, 0, flags, x as u32, y as u32, w, h).request_check()?;
        self.flush();
        Ok(())
    }

    // Remove maximizing attributes
    pub(crate) fn remove_maximize(&self, win: xcb::Window) -> WmCtlResult<()> {
        ewmh::request_change_wm_state(&self.conn, self.screen, win, ewmh::STATE_REMOVE,
            self.conn.WM_ACTION_MAXIMIZE_HORZ(), self.conn.WM_STATE_MAXIMIZED_VERT(), 0).request_check()?;
        Ok(())
    }

    // Get desktop work area
    #[allow(dead_code)]
    pub(crate) fn work_area(&self) -> WmCtlResult<(i32, i32, i32, i32)> {
        let reply = ewmh::get_work_area(&self.conn, self.screen).get_reply()?;
        let geo = reply.work_area().first().unwrap();
        Ok((geo.x() as i32, geo.y() as i32, geo.width() as i32, geo.height() as i32))
    }

    // Get window geometry
    pub(crate) fn win_geometry(&self, win: xcb::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
        let geo = xcb::get_geometry(&self, win).get_reply()?;
        Ok((geo.x() as i32, geo.y() as i32, geo.width() as i32, geo.height() as i32))
    }

    // Get window pid
    #[allow(dead_code)]
    pub(crate) fn win_pid(&self, win: xcb::Window) -> WmCtlResult<u32> {
        let pid = ewmh::get_wm_pid(&self.conn, win).get_reply()?;
        Ok(pid)
    }

    // Get window title
    pub(crate) fn win_title(&self, win: xcb::Window) -> WmCtlResult<String> {
        let name = ewmh::get_wm_name(&self.conn, win).get_reply()?;
        Ok(name.string().to_string())
    }

    /// Get the active window id
    pub(crate) fn active_win(&self) -> WmCtlResult<u32> {
        let active_win = ewmh::get_active_window(&self.conn, self.screen).get_reply()?;
        Ok(active_win)
    }

    /// Identify the taskbar based on sizing to take into account
    pub(crate) fn taskbar(&self) -> WmCtlResult<(u32, u32)> {
        //let taskbar = "xfce4-panel"
        Ok((0, 0))
    }

    /// Get all the windows
    pub(crate) fn windows(&self) -> WmCtlResult<Vec<(u32, String)>> {
        let mut windows = vec![];
        for win_id in ewmh::get_client_list(&self.conn, self.screen).get_reply()?.windows() {

            //  Some window values don't appear to have valid titles and need to be skipped
            if let Ok(name) = self.win_title(*win_id) {
                windows.push((*win_id, name));
            }
        }
        Ok(windows)
    }
}
