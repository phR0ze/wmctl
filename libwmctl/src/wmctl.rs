// Extended Window Manager Hints (EWMH)
// https://specifications.freedesktop.org/wm-spec/latest/
use crate::{WmCtlResult, Position, Win, WinError};
use std::ops::Deref;
use tracing::debug;
use xcb;
use xcb_util::ewmh;

pub(crate) struct WmCtl {
    pub(crate) conn: ewmh::Connection,  // window manager connection
    pub(crate) screen: i32,             // screen number
    pub(crate) full_width: i32,         // screen width
    pub(crate) full_height: i32,        // screen height
    pub(crate) work_width: i32,         // screen width minus possible taskbar
    pub(crate) work_height: i32,        // screen height minus possible taskbar
    pub(crate) taskbar: Position,       // taskbar position
    pub(crate) taskbar_size: i32,       // taskbar reserved space
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

        let mut wmctl = WmCtl{
            conn:  ewmh::Connection::connect(conn).map_err(|(e, _)| e)?,
            screen,
            full_width: full_width as i32,
            full_height: full_height as i32, 
            work_width: Default::default(),
            work_height: Default::default(),
            taskbar: Position::Bottom, // just a default its reset lower down
            taskbar_size: Default::default(),
        };

        // Get the adjusted workspace size i.e. screen full size minus taskbar
        let (_, _, work_width, work_height) = wmctl.work_area()?;
        wmctl.work_width = work_width;
        wmctl.work_height = work_height;

        // Cache the taskbar's position and size
        let (pos, size) = wmctl.taskbar()?;
        wmctl.taskbar = pos;
        wmctl.taskbar_size = size;

        Ok(wmctl)
    }

    /// Get the active window id
    pub(crate) fn active_win(&self) -> WmCtlResult<xcb::Window> {
        let win = ewmh::get_active_window(&self.conn, self.screen).get_reply()?;
        debug!("active_win: id: {}", win);
        Ok(win)
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

        // Only setting the movement flags not the resize flags so width and height won't
        // be used in the final operation.
        let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y;

        // In order to guarantee move will work we must remove maximization states
        self.remove_maximize(win)?;

        // Get the current window position adjusted for decorations
        let (x, y, w, h) = self.win_decorated(win)?;

        // // Compute coordinates based on position
        // let (x, y) = match position {
        //     Position::Center => {
        //         let (mut x, mut y) = ((self.work_width - w)/2, (self.work_height - h)/2);
        //         if x < 0 {
        //             x = 0;
        //         }
        //         if y < 0 {
        //             y = 0;
        //         }
        //         (x, y)
        //     },
        //     Position::Left => (0, 0),
        //     Position::TopLeft => (0, 0), // done
        //     Position::BottomLeft => (0, 0),

        //     // Right: calculate x from right side
        //     Position::Right => (self.work_width - w, y), // y isn't changed
        //     Position::TopRight => (self.work_width - w, 0), // y is zero 
        //     Position::BottomRight => (self.work_width - w, self.work_height - h), // y is calculated from bottom up
        //     Position::Top => (0, 0),
        //     Position::Bottom => (0, 0),
        // };
        // debug!("move_win: id: {}, pos: {}, x: {}, y: {}, w: {}, h: {}", win, position, x, y, w, h);

        // // source: unspecified(0), app(1), pager(2)
        // let (gravity, source) = (0, 2);
        // ewmh::request_move_resize_window(&self.conn, self.screen, win, gravity, source, flags, x as u32, y as u32, 0, 0).request_check()?;
        // self.flush();
        Ok(())
    }

    // Window gemometry accounting for decorations
    pub(crate) fn win_decorated(&self, win: xcb::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
        let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y;

        // 1. get the window's original geometry
        let (x, y, w, h) = self.win_geometry(win)?;

        // 2. shift the window to 0, 0
        ewmh::request_move_resize_window(&self.conn, self.screen, win, 0, 0, flags, 0, 0, 0, 0).request_check()?;
        self.flush();

        // 3. check the x, y offset to determine decoration size
        let g = xcb::get_geometry(&self, win).get_reply()?;
        let (dx, dy) = (g.x() as i32, g.y() as i32);
        debug!("decorations: id: {}, x: {}, y: {}", win, dx, dy);

        // 3. finally shift the window back to where it was accounting for decorations
        let (x, y, w, h) = (x-dx, y-dy, w+dx, h+dy);
        debug!("win_decorated: id: {}, x: {}, y: {}, w: {}, h: {}", win, x, y, w, h);
        ewmh::request_move_resize_window(&self.conn, self.screen, win, 0, 0, flags, x as u32, y as u32, 0, 0).request_check()?;
        self.flush();

        Ok((x, y, w, h))
    }

    // Get window extents
    #[allow(dead_code)]
    pub(crate) fn win_extents(&self, win: xcb::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
        let e = ewmh::get_frame_extents(&self.conn, win).get_reply()?;
        let (l, r, t, b) = (e.left(), e.right(), e.top(), e.bottom());
        debug!("win_extents: id: {}, l: {}, r: {}, t: {}, b: {}", win, l, r, t, b);
        Ok((l as i32, r as i32, t as i32, b as i32))
    }

    // Get window geometry
    pub(crate) fn win_geometry(&self, win: xcb::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
        let g = xcb::get_geometry(&self, win).get_reply()?;
        let (x, y, w, h) = (g.x(), g.y(), g.width(), g.height());
        debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", win, x, y, w, h);
        Ok((x as i32, y as i32, w as i32, h as i32))
    }

    // Get window pid
    #[allow(dead_code)]
    pub(crate) fn win_pid(&self, win: xcb::Window) -> WmCtlResult<u32> {
        let pid = ewmh::get_wm_pid(&self.conn, win).get_reply()?;
        debug!("win_pid: id: {}, pid: {}", win, pid);
        Ok(pid)
    }

    // Get window reservations which is the space the window manager reserved at the edge of the
    // screen for this window e.g. a taskbar at the bottom might have 25pixels reserved at the bottom.
    pub(crate) fn win_reservations(&self, win: xcb::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
        let p = ewmh::get_wm_strut_partial(&self.conn, win).get_reply()?;
        debug!("win_reservations: id: {}, l: {}, r: {}, t: {}, b: {}, {}, {}, {}, {}, {}, {}, {}, {}", win,
            p.left(), p.right(), p.top(), p.bottom(), p.left_start_y(), p.left_end_y(), p.right_start_y(),
            p.right_end_y(), p.top_start_x(), p.top_end_x(), p.bottom_start_x(), p.bottom_end_x());
        Ok((p.left() as i32, p.right() as i32, p.top() as i32, p.bottom() as i32))
    }

    // Get window title
    pub(crate) fn win_title(&self, win: xcb::Window) -> WmCtlResult<String> {
        let reply = ewmh::get_wm_name(&self.conn, win).get_reply()?;
        let title = reply.string().to_string();
        debug!("win_title: id: {}, title: {}", win, title);
        Ok(title)
    }

    // Get window type
    // 390 = app
    // 475 = desktop
    // 476 = panel
    pub(crate) fn win_type(&self, win: xcb::Window) -> WmCtlResult<u32> {
        let reply = ewmh::get_wm_window_type(&self.conn, win).get_reply()?;
        let typ = *reply.atoms().first().unwrap();
        debug!("win_type: id: {}, type: {}", win, typ);
        Ok(typ)
    }

    // Remove maximizing attributes
    pub(crate) fn remove_maximize(&self, win: xcb::Window) -> WmCtlResult<()> {
        debug!("remove_maximize: id: {}", win);
        ewmh::request_change_wm_state(&self.conn, self.screen, win, ewmh::STATE_REMOVE,
            self.conn.WM_ACTION_MAXIMIZE_HORZ(), self.conn.WM_STATE_MAXIMIZED_VERT(), 0).request_check()?;
        Ok(())
    }

    // Get the taskbar window
    pub(crate) fn taskbar(&self) -> WmCtlResult<(Position, i32)> {
        for win in ewmh::get_client_list(&self.conn, self.screen).get_reply()?.windows() {
            if let Ok(geo) = xcb::get_geometry(&self.conn, *win).get_reply() {
                let (x, y, w, h) = (geo.x() as i32, geo.y() as i32, geo.width() as i32, geo.height() as i32);
                if (w == self.full_width as i32 && h == (self.full_height as i32 - self.work_height as i32)) ||
                    (h == self.full_height as i32 && w == (self.full_width as i32 - self.work_width as i32)) {
                    debug!("taskbar: id: {}, x: {}, y: {}, w: {}, h: {}", *win, x, y, w, h);
                    let (l, r, t, b) = self.win_reservations(*win)?;
                    if l > 0 {
                        return Ok((Position::Left, l as i32))
                    } else if r > 0 {
                        return Ok((Position::Right, r as i32))
                    } else if t > 0 {
                        return Ok((Position::Top, t as i32))
                    } else if b > 0 {
                        return Ok((Position::Bottom, b as i32))
                    }
                    return Err(WinError::TaskbarReservationNotFound.into())
                }
            }
        }
        Err(WinError::TaskbarNotFound.into())
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

    // Get desktop work area
    pub(crate) fn work_area(&self) -> WmCtlResult<(i32, i32, i32, i32)> {
        let reply = ewmh::get_work_area(&self.conn, self.screen).get_reply()?;
        let g = reply.work_area().first().unwrap();
        let (x, y, w, h) = (g.x(), g.y(), g.width(), g.height());
        debug!("work_area: x: {}, y: {}, w: {}, h: {}", x, y, w, h);
        Ok((x as i32, y as i32, w as i32, h as i32))
    }
}
