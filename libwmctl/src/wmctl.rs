// Extended Window Manager Hints (EWMH)
// https://specifications.freedesktop.org/wm-spec/latest/

// XCB is a library implementing the client-side of the X11 display server protocol. The project
// was created with the aim of replacing Xlib. It was designed as a smaller, modernized
// replacement of Xlib. Using XCB programs don't need to implement the X protocol layer.
// 
// XCB allows you to sends a number of requests then ask for the replies later allowing for the
// server to do the work more efficiently.
// https://xcb.freedesktop.org/tutorial/

use crate::{WmCtlResult, WinPosition, WmCtlError, WinClass, WinState, WinType};
use std::{str, ops::Deref};
use tracing::{trace, debug};

use x11rb::{
    atom_manager,
    connection::Connection,
    protocol::xproto::{ConnectionExt as _, self, *},
    wrapper::ConnectionExt as _,
    xcb_ffi::XCBConnection,
};

// A collection of the atoms we will need.
atom_manager! {
    pub(crate) AtomCollection: AtomCollectionCookie {
        _NET_ACTIVE_WINDOW,
        _NET_NUMBER_OF_DESKTOPS,
        _NET_WM_DESKTOP,
        _NET_WM_NAME,
        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_DIALOG,
        _NET_WM_WINDOW_TYPE_DOCK,
        _NET_WM_WINDOW_TYPE_DROPDOWN_MENU,
        _NET_WM_WINDOW_TYPE_MENU,
        _NET_WM_WINDOW_TYPE_NORMAL,
        _NET_WM_WINDOW_TYPE_NOTIFICATION,
        _NET_WM_WINDOW_TYPE_POPUP_MENU,
        _NET_WM_WINDOW_TYPE_SPLASH,
        _NET_WM_WINDOW_TYPE_TOOLBAR,
        _NET_WM_WINDOW_TYPE_TOOLTIP,
        _NET_WM_WINDOW_TYPE_UTILITY,
        UTF8_STRING,
    }
}

// Window Manager controller
pub(crate) struct WmCtl
{
    pub(crate) xcb: XCBConnection,     // window manager connection
    pub(crate) screen: usize,           // screen number
    pub(crate) root: u32,               // root window id
    pub(crate) width: u16,              // screen width
    pub(crate) height: u16,             // screen height
    pub(crate) atoms: AtomCollection,   // atom cache
}

// impl Deref for WmCtl
// {
// 	type Target = RustConnection;

// 	fn deref(&self) -> &Self::Target {
// 		&self.conn
// 	}
// }

// Connect to the X11 server
impl WmCtl
{
    pub(crate) fn connect() -> WmCtlResult<Self> {
        let (conn, screen) = XCBConnection::connect(None)?;
        let atoms = AtomCollection::new(&conn)?.reply()?;

        // Get the screen size
        let (width, height, root) = {
            let screen = &conn.setup().roots[screen];
            (screen.width_in_pixels, screen.height_in_pixels, screen.root)
        };

        debug!("connect: screen: {}, root: {}, w: {}, h: {}", screen, root, width, height);
        Ok(Self{xcb: conn, screen, root, width, height, atoms})
    }

    /// Get the active window id
    pub(crate) fn active_win(&self) -> WmCtlResult<u32> {
        let win = 0u32;
        //let win = ewmh::get_active_window(&self.conn, self.screen).get_reply()?;

        // let reply = conn.get_property(false, root, atoms._NET_ACTIVE_WINDOW, AtomEnum::WINDOW, 0, std::u32::MAX)?.reply()?;
        // let win = reply.value.first().unwrap();
        // println!("ACTIVE: {}", win);
        // let reply = conn.get_property(false, *win as u32, AtomEnum::WM_NAME, AtomEnum::STRING, 0, std::u32::MAX)?.reply()?;
        // let name = str::from_utf8(&reply.value)?.to_string();
        // println!("NAME: {}", name);

        debug!("active_win: {}", win);
        Ok(win)
    }

    // Check if a composit manager is running
    pub(crate) fn composite_manager(&self) -> WmCtlResult<bool> {
        let atom = format!("_NET_WM_CM_S{}", self.screen);
        let atom = self.xcb.intern_atom(false, atom.as_bytes())?.reply()?.atom;
        let reply = self.xcb.get_selection_owner(atom)?.reply()?;
        let result = reply.owner != x11rb::NONE;
        debug!("composite_manager: {}", result);
        Ok(result)
    }

    // Get number of desktops
    pub(crate) fn desktops(&self) -> WmCtlResult<u32> {
        let reply = self.xcb.get_property(false, self.root, self.atoms._NET_NUMBER_OF_DESKTOPS, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let num = match reply.value32().and_then(|mut x| x.next()) {
            Some(value) => value,
            None => Err(WmCtlError::InvalidDesktopNum)?,
        };
        debug!("desktops: {}", num);
        Ok(num)
    }

    // Get window attribrtes
    pub(crate) fn win_attributes(&self, win: xproto::Window) -> WmCtlResult<(WinClass, WinState)> {
        let attr = self.xcb.get_window_attributes(win)?.reply()?;
        debug!("win_attributes: id: {}, class: {:?}, state: {:?}", win, attr.class, attr.map_state);
        Ok((WinClass::from(attr.class.into())?, WinState::from(attr.map_state.into())?))
    }

    // Get window desktop
    pub(crate) fn win_desktop(&self, win: xproto::Window) -> WmCtlResult<u32> {
        // let reply = self.xcb.get_property(false, win, self.atoms._NET_WM_DESKTOP, AtomEnum::ANY, 0, u32::MAX)?.reply()?;
        // let desktop = match reply.value32().and_then(|mut x| x.next()) {
        //     Some(value) => value,
        //     None => 5, 
        //     //None => Err(WmCtlError::InvalidDesktopNum)?,
        // };
        // debug!("win_desktop: id: {}, desktop: {}", win, desktop);
        // Ok(desktop)
        Ok(5)
    }

    // Get window geometry
    pub(crate) fn win_geometry(&self, win: xproto::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
        let g = self.xcb.get_geometry(win)?.reply()?;
        let (x, y, w, h) = (g.x, g.y, g.width, g.height);
        debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", win, x, y, w, h);
        Ok((x as i32, y as i32, w as i32, h as i32))
    }

    // Get window name
    pub(crate) fn win_name(&self, win: xproto::Window) -> WmCtlResult<String> {

        // First try the _NET_WM_NAME
        let reply = self.xcb.get_property(false, win, self.atoms._NET_WM_NAME, self.atoms.UTF8_STRING, 0, u32::MAX)?.reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    trace!("win_name: using _NET_WM_NAME for: {}", value);
                    return Ok(value.to_owned())
                }
            }
        }

        // Fall back on the WM_NAME
        let reply = self.xcb.get_property(false, win, AtomEnum::WM_NAME, AtomEnum::STRING, 0, u32::MAX)?.reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    trace!("win_name: using WM_NAME for: {}", value);
                    return Ok(value.to_owned())
                }
            }
        }

        // No valid name was found
        Err(WmCtlError::WinNameNotFound.into())
    }

    // Get window type
    pub(crate) fn win_type(&self, win: xproto::Window) -> WmCtlResult<WinType> {
        let reply = self.xcb.get_property(false, win, self.atoms._NET_WM_WINDOW_TYPE, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
        let typ = match reply.value32().and_then(|mut x| x.next()) {
            Some(typ) => typ,
            None => Err(WmCtlError::WinTypeNotFound)?,
        };
        //println!("DataType NET: {:?}", AtomEnum::from(reply.type_ as u8));
        // if reply.type_ == x11rb::NONE {
        //     println!("type: NONE");
        // } else if reply.type_ == AtomEnum::ATOM.into() {
        //     println!("type: ATOM");
        // }

        let typ = WinType::from(&self.atoms, typ)?;
        debug!("win_type: id: {}, type: {:?}", win, typ);
        Ok(typ)
    }

    // // Get window type
    // pub(crate) fn win_type(&self, win: xproto::Window) -> WmCtlResult<()> {
    //     let reply = ewmh::get_wm_window_type(&self.conn, win).get_reply()?;
    //     let typ = WinType::from(&self.conn, reply)?;
    //     debug!("win_type: id: {}, type: {}", win, typ);
    //     Ok(())
    // }

    // // Move and resize window
    // pub(crate) fn move_and_resize(&self, win: xcb::Window, x: i32, y: i32, w: i32, h: i32) -> WmCtlResult<()> {
    //     //let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y | ewmh::MOVE_RESIZE_WINDOW_WIDTH | ewmh::MOVE_RESIZE_WINDOW_HEIGHT;
    //     let (x, y, w, h) = (500, 00, 1000, 1000);
    //     let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y;
    //     ewmh::request_move_resize_window(&self.conn, self.screen, win, 0, 0, flags, x as u32, y as u32, w as u32, h as u32).request_check()?;
    //     self.flush();
    //     Ok(())
    // }

    // // Move window
    // pub(crate) fn move_win(&self, win: xcb::Window, position: Position) -> WmCtlResult<()> {

    //     // Only setting the movement flags not the resize flags so width and height won't
    //     // be used in the final operation.
    //     let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y;

    //     // In order to guarantee move will work we must remove maximization states
    //     self.remove_maximize(win)?;

    //     // Get the current window position adjusted for decorations
    //     let (x, y, w, h) = self.win_decorated(win)?;

    //     // // Compute coordinates based on position
    //     // let (x, y) = match position {
    //     //     Position::Center => {
    //     //         let (mut x, mut y) = ((self.work_width - w)/2, (self.work_height - h)/2);
    //     //         if x < 0 {
    //     //             x = 0;
    //     //         }
    //     //         if y < 0 {
    //     //             y = 0;
    //     //         }
    //     //         (x, y)
    //     //     },
    //     //     Position::Left => (0, 0),
    //     //     Position::TopLeft => (0, 0), // done
    //     //     Position::BottomLeft => (0, 0),

    //     //     // Right: calculate x from right side
    //     //     Position::Right => (self.work_width - w, y), // y isn't changed
    //     //     Position::TopRight => (self.work_width - w, 0), // y is zero 
    //     //     Position::BottomRight => (self.work_width - w, self.work_height - h), // y is calculated from bottom up
    //     //     Position::Top => (0, 0),
    //     //     Position::Bottom => (0, 0),
    //     // };
    //     // debug!("move_win: id: {}, pos: {}, x: {}, y: {}, w: {}, h: {}", win, position, x, y, w, h);

    //     // // source: unspecified(0), app(1), pager(2)
    //     // let (gravity, source) = (0, 2);
    //     // ewmh::request_move_resize_window(&self.conn, self.screen, win, gravity, source, flags, x as u32, y as u32, 0, 0).request_check()?;
    //     // self.flush();
    //     Ok(())
    // }

    // // Window gemometry accounting for decorations
    // pub(crate) fn win_decorated(&self, win: xcb::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
    //     let flags = ewmh::MOVE_RESIZE_WINDOW_X | ewmh::MOVE_RESIZE_WINDOW_Y;

    //     // 1. get the window's original geometry
    //     let (x, y, w, h) = self.win_geometry(win)?;

    //     // 2. shift the window to 0, 0
    //     ewmh::request_move_resize_window(&self.conn, self.screen, win, 0, 0, flags, 0, 0, 0, 0).request_check()?;
    //     self.flush();

    //     // 3. check the x, y offset to determine decoration size
    //     let g = xcb::get_geometry(&self, win).get_reply()?;
    //     let (dx, dy) = (g.x() as i32, g.y() as i32);
    //     debug!("decorations: id: {}, x: {}, y: {}", win, dx, dy);

    //     // 3. finally shift the window back to where it was accounting for decorations
    //     let (x, y, w, h) = (x-dx, y-dy, w+dx, h+dy);
    //     debug!("win_decorated: id: {}, x: {}, y: {}, w: {}, h: {}", win, x, y, w, h);
    //     ewmh::request_move_resize_window(&self.conn, self.screen, win, 0, 0, flags, x as u32, y as u32, 0, 0).request_check()?;
    //     self.flush();

    //     Ok((x, y, w, h))
    // }

    // // Get window extents
    // #[allow(dead_code)]
    // pub(crate) fn win_extents(&self, win: xcb::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
    //     let e = ewmh::get_frame_extents(&self.conn, win).get_reply()?;
    //     let (l, r, t, b) = (e.left(), e.right(), e.top(), e.bottom());
    //     debug!("win_extents: id: {}, l: {}, r: {}, t: {}, b: {}", win, l, r, t, b);
    //     Ok((l as i32, r as i32, t as i32, b as i32))
    // }

    // Get window geometry
    // pub(crate) fn win_geometry(&self, win: u32) -> WmCtlResult<(i32, i32, i32, i32)> {
    //     let g = self.conn.get_geometry(win)?.reply()?;
    //     let (x, y, w, h) = (g.x, g.y, g.width, g.height);
    //     debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", win, x, y, w, h);
    //     Ok((x as i32, y as i32, w as i32, h as i32))
    // }

    // // Get window pid
    // #[allow(dead_code)]
    // pub(crate) fn win_pid(&self, win: xcb::Window) -> WmCtlResult<u32> {
    //     let pid = ewmh::get_wm_pid(&self.conn, win).get_reply()?;
    //     debug!("win_pid: id: {}, pid: {}", win, pid);
    //     Ok(pid)
    // }

    // // Get window reservations which is the space the window manager reserved at the edge of the
    // // screen for this window e.g. a taskbar at the bottom might have 25pixels reserved at the bottom.
    // pub(crate) fn win_reservations(&self, win: xcb::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
    //     let p = ewmh::get_wm_strut_partial(&self.conn, win).get_reply()?;
    //     debug!("win_reservations: id: {}, l: {}, r: {}, t: {}, b: {}, {}, {}, {}, {}, {}, {}, {}, {}", win,
    //         p.left(), p.right(), p.top(), p.bottom(), p.left_start_y(), p.left_end_y(), p.right_start_y(),
    //         p.right_end_y(), p.top_start_x(), p.top_end_x(), p.bottom_start_x(), p.bottom_end_x());
    //     Ok((p.left() as i32, p.right() as i32, p.top() as i32, p.bottom() as i32))
    // }

    // Get window name
    // pub(crate) fn win_name(&self, win: u32) -> WmCtlResult<String> {
    //     let reply = self.conn.get_property(false, win, AtomEnum::WM_NAME, AtomEnum::STRING, 0, std::u32::MAX)?.reply()?;
    //     let name = str::from_utf8(&reply.value)?.to_string();
    //     debug!("win_name: id: {}, name: {}", win, name);
    //     Ok(name)
    // }
/*  */
    // // Get window type
    // // 390 = app
    // // 475 = desktop
    // // 476 = panel
    // pub(crate) fn win_type(&self, win: xcb::Window) -> WmCtlResult<u32> {
    //     let reply = ewmh::get_wm_window_type(&self.conn, win).get_reply()?;
    //     let typ = *reply.atoms().first().unwrap();
    //     debug!("win_type: id: {}, type: {}", win, typ);
    //     Ok(typ)
    // }

    // // Remove maximizing attributes
    // pub(crate) fn remove_maximize(&self, win: xcb::Window) -> WmCtlResult<()> {
    //     debug!("remove_maximize: id: {}", win);
    //     ewmh::request_change_wm_state(&self.conn, self.screen, win, ewmh::STATE_REMOVE,
    //         self.conn.WM_ACTION_MAXIMIZE_HORZ(), self.conn.WM_STATE_MAXIMIZED_VERT(), 0).request_check()?;
    //     Ok(())
    // }

    // // Get the taskbar window
    // pub(crate) fn taskbar(&self) -> WmCtlResult<(Position, i32)> {
    //     for win in ewmh::get_client_list(&self.conn, self.screen).get_reply()?.windows() {
    //         if let Ok(geo) = xcb::get_geometry(&self.conn, *win).get_reply() {
    //             let (x, y, w, h) = (geo.x() as i32, geo.y() as i32, geo.width() as i32, geo.height() as i32);
    //             if (w == self.full_width as i32 && h == (self.full_height as i32 - self.work_height as i32)) ||
    //                 (h == self.full_height as i32 && w == (self.full_width as i32 - self.work_width as i32)) {
    //                 debug!("taskbar: id: {}, x: {}, y: {}, w: {}, h: {}", *win, x, y, w, h);
    //                 let (l, r, t, b) = self.win_reservations(*win)?;
    //                 if l > 0 {
    //                     return Ok((Position::Left, l as i32))
    //                 } else if r > 0 {
    //                     return Ok((Position::Right, r as i32))
    //                 } else if t > 0 {
    //                     return Ok((Position::Top, t as i32))
    //                 } else if b > 0 {
    //                     return Ok((Position::Bottom, b as i32))
    //                 }
    //                 return Err(WinError::TaskbarReservationNotFound.into())
    //             }
    //         }
    //     }
    //     Err(WinError::TaskbarNotFound.into())
    // }

    /// Get all the windows
    /// https://tronche.com/gui/x/xlib/
    /// 
    /// Window Attributes
    /// https://tronche.com/gui/x/xlib/window/attributes/
    /// 
    /// * INPUT_OUTPUT windows have a border width of zero or more pixels and share the same root
    ///   window loaded from screen.root. INPUT_ONLY windows, which are invisible, are used for controlling input
    /// * INPUT_ONLY windows are invisible and used for controlling input events in situations where an InputOutput
    ///   window is unnecessary and cannot have INPUT_OUTPUT windows as inferiors.
    /// [(id, name, type, class, state, (x, y, w, h))]
    pub(crate) fn windows(&self) -> WmCtlResult<Vec<(u32, String, WinType, WinClass, WinState, (u32, u32, u32, u32))>> {
        let mut windows = vec![];
        for win in self.xcb.query_tree(self.root)?.reply()?.children {

            // Filter out windows without a valid window type
            let typ = match self.win_type(win) {
                Ok(typ) => typ,
                Err(_) => WinType::Invalid,
            };

            // Filter out windows that don't have valid sizes
            // Often windows used for input only or tracking will have odd dimentions like 1x1
            let (x, y, w, h) = match self.win_geometry(win) {
                Ok((x, y, w, h)) => {
                    if w < 1 || h < 1 {
                        //continue;
                        (0, 0, 0, 0)
                    } else {
                        (x, y, w, h)
                    }
                },
                //Err(_) => continue,
                Err(_) => (0, 0, 0, 0),
            };

            // Use empty string for windows with invalid names
            let name = match self.win_name(win) {
                Ok(name) => name,
                Err(_) => "".to_string(),
            };

            // Filter out windows that are INPUT_ONLY
            let (class, state) = self.win_attributes(win)?;
            // if class == WindowClass::INPUT_ONLY {
            //     continue;
            // }

            windows.push((win, name, typ, class, state, (x as u32, y as u32, w as u32, h as u32)));
        }

        // // Setup requests to get all window attributes and geometries
        // let tree = self.xcb.query_tree(self.root)?.reply()?;
        // let mut cookies = Vec::with_capacity(tree.children.len());
        // for win in tree.children {
        //     //let attr = self.xcb.get_window_attributes(win)?;
        //     cookies.push((win, attr));
        // }

        // // Now process the replies
        // for (win, attr, geom) in cookies {
        //     let (attr, geom) = (attr.reply(), geom.reply());

        //     // Filter out windows that are not valid
        //     if attr.is_err() || geom.is_err() {
        //         continue;
        //     }
        //     let (attr, geom) = (attr.unwrap(), geom.unwrap());

        //     let (x, y, w, h) = self.win_geometry(win)?;

        //     // xcb name
        //     let name = self.win_name(win)?;

        //     // if format!("{:?}", attr.map_state) == "VIEWABLE" {
        //     if name != "" {
        //         debug!("windows: id: {:<9} pos: {:<11} {:<10} {}", win, format!("{}x{}", geom.x, geom.y), format!("{}x{}", geom.width, geom.height), name);
        //     }
        // }

        Ok(windows)
    }

    // // Get desktop work area
    // pub(crate) fn work_area(&self) -> WmCtlResult<(i32, i32, i32, i32)> {
    //     let reply = ewmh::get_work_area(&self.conn, self.screen).get_reply()?;
    //     let g = reply.work_area().first().unwrap();
    //     let (x, y, w, h) = (g.x(), g.y(), g.width(), g.height());
    //     debug!("work_area: x: {}, y: {}, w: {}, h: {}", x, y, w, h);
    //     Ok((x as i32, y as i32, w as i32, h as i32))
    // }

    // EWMH helper methods
    // https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints
    // ---------------------------------------------------------------------------------------------

    // // Window manager protocols
    // pub(crate) fn _wm_protocols(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"WM_PROTOCOLS")?.reply()?.atom)
    // }

    // // Lists all the EWWH protocols supported by this WM
    // pub(crate) fn _supported(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_SUPPORTED")?.reply()?.atom)
    // }

    // // Indicates the number of virtual desktops
    // pub(crate) fn _num_desktops(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_NUMBER_OF_DESKTOPS")?.reply()?.atom)
    // }

    // // Defines the common size of all desktops
    // pub(crate) fn _desktop_geometry(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_DESKTOP_GEOMETRY")?.reply()?.atom)
    // }

    // // Defines the top left corner of each desktop
    // pub(crate) fn _desktop_viewport(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_DESKTOP_VIEWPORT")?.reply()?.atom)
    // }

    // // Get/set the currently active window
    // pub(crate) fn _active_win(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_ACTIVE_WINDOW")?.reply()?.atom)
    // }

    // // Contains a geometry for each desktop
    // pub(crate) fn _work_area(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_WORKAREA")?.reply()?.atom)
    // }

    // // Give the window of the active WM
    // pub(crate) fn _win_manger(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_SUPPORTING_WM_CHECK")?.reply()?.atom)
    // }

    // // Interactively resize and application window
    // pub(crate) fn _wm_moveresize(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_WM_MOVERESIZE")?.reply()?.atom)
    // }

    // // Immediately resize an application window
    // pub(crate) fn _moveresize_win(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_MOVERESIZE_WINDOW")?.reply()?.atom)
    // }

    // // The left, right, top and bottom frame sizes
    // pub(crate) fn _frame_extents(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_REQUEST_FRAME_EXTENTS")?.reply()?.atom)
    // }
    
    // // Title of the window
    // pub(crate) fn _win_name(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?.atom)
    // }

    // // The window title as shown by the WM
    // pub(crate) fn _win_visiable_name(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_WM_VISIBLE_NAME")?.reply()?.atom)
    // }

    // // The icon title
    // pub(crate) fn _win_icon_name(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_WM_ICON_NAME")?.reply()?.atom)
    // }
    
    // // The icon title as shown by the WM
    // pub(crate) fn _win_visiable_icon_name(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_WM_VISIBLE_ICON_NAME")?.reply()?.atom)
    // }

    // // The desktop the window is in
    // pub(crate) fn _win_desktop(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_WM_DESKTOP")?.reply()?.atom)
    // }

    // // The functional type of the window
    // pub(crate) fn _win_type(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE")?.reply()?.atom)
    // }

    // // The current window state
    // pub(crate) fn _win_state(&self) -> WmCtlResult<xproto::Atom> {
    //     Ok(self.conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom)
    // }

    // conn.intern_atom(false, b"_NET_WM_STATE_ABOVE")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_STATE_STICKY")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_STATE_MODAL")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_STATE_FULLSCREEN")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_STRUT_PARTIAL")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_NORMAL")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_DIALOG")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_UTILITY")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_TOOLBAR")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_SPLASH")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_MENU")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_DROPDOWN_MENU")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_POPUP_MENU")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_TOOLTIP")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_NOTIFICATION")?.reply()?.atom,
    // conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_DOCK")?.reply()?.atom,
}
