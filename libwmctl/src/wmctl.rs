use crate::{WmCtlResult, Position, Win, WinError};
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
    pub(crate) taskbar: Win,            // taskbar geometry
}

impl Deref for WmCtl {
	type Target = xcb::Connection;

	fn deref(&self) -> &Self::Target {
		&self.conn
	}
}

// Connect to the X11 server
impl WmCtl {
    pub(crate) fn connect() -> WmCtlResult<WmCtl> {
        let (conn, screen) = xcb::Connection::connect(None)?;

        // Get the full screen size
        let (full_width, full_height) = {
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

        // Get the taskbar window
        let mut taskbar = Win {x: 0, y: 0, w: 0, h: 0};
        for win in ewmh::get_client_list(&conn, screen).get_reply()?.windows() {
            if let Ok(geo) = xcb::get_geometry(&conn, *win).get_reply() {
                let (x, y, w, h) = (geo.x() as i32, geo.y() as i32, geo.width() as i32, geo.height() as i32);
                if w == full_width as i32 && h == (full_height as i32 - work_height as i32) {
                    taskbar = Win{x, y, w, h};
                    break;
                } else if h == full_height as i32 && w == (full_width as i32 - work_width as i32) {
                    taskbar = Win{x, y, w, h};
                    break;
                }
            }
        }
        if taskbar.w == 0 && taskbar.h == 0 {
            return Err(WinError::TaskbarNotFound.into())
        }

        Ok(WmCtl{
            conn,
            screen,
            full_width: full_width as i32,
            full_height: full_height as i32, 
            work_width: work_width as i32,
            work_height: work_height as i32,
            taskbar,
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
        let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y;

        // Get the current window position. Since we're not including the ...WIDTH and ...HEIGHT flags
        // we are ignoring the w, h values returned from win_geometry
        let (x, y, w, h) = self.win_geometry(win)?;

        println!("full w: {}, h: {}", self.full_width, self.full_height);
        println!("work w: {}, h: {}", self.work_width, self.work_height);


        // Compute coordinates based on position
        println!("1: x: {}, y: {}, w: {}, h: {}", x, y, w, h);
        let (x, y) = match position {
            Position::Center => {
                let (mut x, mut y) = ((self.work_width - w)/2, (self.work_height - h)/2);
                if x < 0 {
                    x = 0;
                }
                if y < 0 {
                    y = 0;
                }
                (x, y)
            },
            Position::Left => (0, 0),
            Position::Right => (self.work_width - w, self.work_height - h),
            Position::Top => (0, 0),
            Position::Bottom => (0, 0),
            Position::TopLeft => (0, 0),
            Position::TopRight => (self.work_width - w, 0),
            Position::BottomLeft => (0, 0),
            Position::BottomRight => (0, 0),
        };
        println!("2: x: {}, y: {}, w: {}, h: {}", x, y, w, h);

        ewmh::request_move_resize_window(&self.conn, self.screen, win, 0, 0, flags, x as u32, y as u32, w as u32, h as u32).request_check()?;
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

    // Get window type
    // 390 = app
    // 475 = desktop
    // 476 = panel
    pub(crate) fn win_type(&self, win: xcb::Window) -> WmCtlResult<u32> {
        let reply = ewmh::get_wm_window_type(&self.conn, win).get_reply()?;
        Ok(*reply.atoms().first().unwrap())
    }

    /// Get the active window id
    pub(crate) fn active_win(&self) -> WmCtlResult<xcb::Window> {
        Ok(ewmh::get_active_window(&self.conn, self.screen).get_reply()?)
    }

    /// Get all the windows
    pub(crate) fn windows(&self) -> WmCtlResult<Vec<(u32, String)>> {
        let mut windows = vec![];
        for win in ewmh::get_client_list(&self.conn, self.screen).get_reply()?.windows() {

            //  Some window values don't appear to have valid titles and need to be skipped
            if let Ok(name) = self.win_title(*win) {
                windows.push((*win, name));
            }
        }
        Ok(windows)
    }
}
