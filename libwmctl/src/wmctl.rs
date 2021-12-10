// Extended Window Manager Hints (EWMH)
// https://specifications.freedesktop.org/wm-spec/latest/
//
// The EWHM spec builds on the lower level Inter Client Communication Conventions Manual (ICCCM)
// to define interactions between window managers, compositing managers and applications.
// 
// Root Window Properties
// https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html
//
// The EWMH spec defines a number of properties that EWHM compliant window managers will maintain
// and return to clients requesting information.
use crate::{WmCtlResult, WinPosition, WmCtlError, WinClass, WinState, WinType};
use std::{str, ops::Deref};
use tracing::{trace, debug};

use x11rb::{
    atom_manager,
    connection::Connection,
    protocol::xproto::{ConnectionExt as _, self, *},
    wrapper::ConnectionExt as _,
    //xcb_ffi::XCBConnection,
    rust_connection::RustConnection,
};

// A collection of the atoms we will need.
atom_manager! {
    pub(crate) AtomCollection: AtomCollectionCookie {
        _NET_ACTIVE_WINDOW,
        _NET_CLIENT_LIST,
        _NET_NUMBER_OF_DESKTOPS,
        _NET_WORKAREA,
        _NET_WM_DESKTOP,
        _NET_WM_NAME,
        _NET_WM_VISIBLE_NAME,
        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_COMBO,
        _NET_WM_WINDOW_TYPE_DESKTOP,
        _NET_WM_WINDOW_TYPE_DIALOG,
        _NET_WM_WINDOW_TYPE_DND,
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

// Window Manager control provides a simplified access layer to the EWMH functions exposed
// through the x11 libraries.
pub(crate) struct WmCtl
{
    pub(crate) conn: RustConnection,     // x11 connection
    pub(crate) screen: usize,           // screen number
    pub(crate) root: u32,               // root window id
    pub(crate) width: u16,              // screen width
    pub(crate) height: u16,             // screen height
    pub(crate) work_width: u16,         // screen height
    pub(crate) work_height: u16,        // screen height
    pub(crate) atoms: AtomCollection,   // atom cache
}

impl Deref for WmCtl {
	type Target = RustConnection;

	fn deref(&self) -> &Self::Target {
		&self.conn
	}
}

impl WmCtl
{
    pub(crate) fn connect() -> WmCtlResult<Self> {
        let (conn, screen) = x11rb::connect(None)?;
        let atoms = AtomCollection::new(&conn)?.reply()?;

        // Get the screen size
        let (width, height, root) = {
            let screen = &conn.setup().roots[screen];
            (screen.width_in_pixels, screen.height_in_pixels, screen.root)
        };

        // Create the window manager object
        let mut wmctl = WmCtl{
            conn, screen, root, width, height,
            work_width: Default::default(),
            work_height: Default::default(),
            atoms
        };

        // Get the work area
        let (width, height) = wmctl.work_area()?;
        wmctl.work_width = width;
        wmctl.work_height = height;

        debug!("connect: screen: {}, root: {}, w: {}, h: {}", screen, root, width, height);
        Ok(wmctl)
    }

    // Get the active window id
    // Defined as: _NET_ACTIVE_WINDOW, WINDOW/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_ACTIVE_WINDOW`
    // request message with a `AtomEnum::WINDOW` type response and we can use the `reply.value32()` accessor to
    // retrieve the value.
    pub(crate) fn active_win(&self) -> WmCtlResult<u32> {
        let reply = self.get_property(false, self.root, self.atoms._NET_ACTIVE_WINDOW, AtomEnum::WINDOW, 0, u32::MAX)?.reply()?;
        let win = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_ACTIVE_WINDOW".to_owned()))?;
        debug!("active_win: {}", win);
        Ok(win)
    }

    // Check if a composit manager is running
    // Defined as: _NET_WM_CM_Sn 
    // For each screen the compositing manager manages they MUST acquire ownership of a selection named _NET_WM_CM_Sn,
    // where the suffix `n` is the screen number.
    pub(crate) fn composite_manager(&self) -> WmCtlResult<bool> {
        let atom = format!("_NET_WM_CM_S{}", self.screen);
        let atom = self.intern_atom(false, atom.as_bytes())?.reply()?.atom;
        let reply = self.get_selection_owner(atom)?.reply()?;
        let result = reply.owner != x11rb::NONE;
        debug!("composite_manager: {}", result);
        Ok(result)
    }

    // Get number of desktops
    // Defined as: _NET_NUMBER_OF_DESKTOPS, CARDINAL/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_NUMBER_OF_DESKTOPS`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the value.
    pub(crate) fn desktops(&self) -> WmCtlResult<u32> {
        let reply = self.get_property(false, self.root, self.atoms._NET_NUMBER_OF_DESKTOPS, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let num = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_NUMBER_OF_DESKTOPS".to_owned()))?;
        debug!("desktops: {}", num);
        Ok(num)
    }

    // Get desktop work area
    // Defined as: _NET_WORKAREA, x, y, width, height CARDINAL[][4]/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WORKAREA`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be 4 for each desktop as defined (x, y, width, height).
    pub(crate) fn work_area(&self) -> WmCtlResult<(u16, u16)> {
        let reply = self.get_property(false, self.root, self.atoms._NET_WORKAREA, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let mut values = reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA".to_owned()))?.skip(2);
        let w = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA width".to_owned()))?;
        let h = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA height".to_owned()))?;
        debug!("work_area: w: {}, h: {}", w, h);
        Ok((w as u16, h as u16))
    }

    // Get window attribrtes
    pub(crate) fn win_attributes(&self, win: xproto::Window) -> WmCtlResult<(WinClass, WinState)> {
        let attr = self.get_window_attributes(win)?.reply()?;
        debug!("win_attributes: id: {}, class: {:?}, state: {:?}", win, attr.class, attr.map_state);
        Ok((WinClass::from(attr.class.into())?, WinState::from(attr.map_state.into())?))
    }

    // Get window desktop
    // Defined as: _NET_WM_DESKTOP desktop, CARDINAL/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_DESKTOP`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will.
    pub(crate) fn win_desktop(&self, win: xproto::Window) -> WmCtlResult<i32> {
        let reply = self.get_property(false, win, self.atoms._NET_WM_DESKTOP, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let desktop = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_WM_DESKTOP".to_owned()))?;
        debug!("win_desktop: id: {}, desktop: {}", win, desktop);
        Ok(desktop as i32)
    }

    // Get window geometry
    pub(crate) fn win_geometry(&self, win: xproto::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
        let g = self.get_geometry(win)?.reply()?;
        let (x, y, w, h) = (g.x, g.y, g.width, g.height);
        debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", win, x, y, w, h);
        Ok((x as i32, y as i32, w as i32, h as i32))
    }

    // Get window name
    // Defined as: _NET_WM_NAME, UTF8_STRING
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_NAME`
    // request message with a `AtomEnum::UTF8_STRING` type response and we can use the `reply.value` accessor to
    // retrieve the value.
    pub(crate) fn win_name(&self, win: xproto::Window) -> WmCtlResult<String> {

        // First try the _NET_WM_VISIBLE_NAME
        let reply = self.get_property(false, win, self.atoms._NET_WM_VISIBLE_NAME, self.atoms.UTF8_STRING, 0, u32::MAX)?.reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using _NET_WM_VISIBLE_NAME for: {}", value);
                    return Ok(value.to_owned())
                }
            }
        }

        // Next try the _NET_WM_NAME
        let reply = self.get_property(false, win, self.atoms._NET_WM_NAME, self.atoms.UTF8_STRING, 0, u32::MAX)?.reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using _NET_WM_NAME for: {}", value);
                    return Ok(value.to_owned())
                }
            }
        }

        // Fall back on the WM_NAME
        let reply = self.get_property(false, win, AtomEnum::WM_NAME, AtomEnum::STRING, 0, u32::MAX)?.reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using WM_NAME for: {}", value);
                    return Ok(value.to_owned())
                }
            }
        }

        // No valid name was found
        Err(WmCtlError::PropertyNotFound("_NET_WM_NAME | _WM_NAME".to_owned()).into())
    }

    // Get window type
    // Defined as: _NET_WM_WINDOW_TYPE, ATOM[]/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_WINDOW_TYPE`
    // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
    // retrieve the value.
    pub(crate) fn win_type(&self, win: xproto::Window) -> WmCtlResult<WinType> {
        let reply = self.get_property(false, win, self.atoms._NET_WM_WINDOW_TYPE, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
        let typ = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_WM_WINDOW_TYPE".to_owned()))?;
        let typ = WinType::from(&self.atoms, typ)?;
        debug!("win_type: id: {}, type: {:?}", win, typ);
        Ok(typ)
    }

    // Get client windows
    // Defined as: _NET_CLIENT_LIST, WINDOW[]/32 
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_CLIENT_LIST`
    // request message with a `AtomEnum::WINDOW` type response and we can use the `reply.value32()` accessor to
    // retrieve the values
    pub(crate) fn windows(&self) -> WmCtlResult<Vec<u32>> {
        let mut windows = vec![];
        let reply = self.get_property(false, self.root, self.atoms._NET_CLIENT_LIST, AtomEnum::WINDOW, 0, u32::MAX)?.reply()?;
        for win in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_CLIENT_LIST".to_owned()))? {
            windows.push(win)
        }
        Ok(windows)
    }

    /// Get all X windows
    /// https://tronche.com/gui/x/xlib/
    /// 
    /// Window Attributes
    /// https://tronche.com/gui/x/xlib/window/attributes/
    /// 
    /// * INPUT_OUTPUT windows have a border width of zero or more pixels and share the same root
    ///   window loaded from screen.root. INPUT_ONLY windows, which are invisible, are used for controlling input
    /// * INPUT_ONLY windows are invisible and used for controlling input events in situations where an InputOutput
    ///   window is unnecessary and cannot have INPUT_OUTPUT windows as inferiors.
    pub(crate) fn all_windows(&self) -> WmCtlResult<Vec<u32>> {
        let mut windows = vec![];
        let tree = self.query_tree(self.root)?.reply()?;
        for win in tree.children {

            // // Filter out windows without a valid window type
            // let typ = match self.win_type(win) {
            //     Ok(typ) => typ,
            //     Err(_) => WinType::Invalid,
            // };

            // // Filter out windows that don't have valid sizes
            // // Often windows used for input only or tracking will have odd dimentions like 1x1
            // let (x, y, w, h) = match self.win_geometry(win) {
            //     Ok((x, y, w, h)) => {
            //         if w < 1 || h < 1 {
            //             //continue;
            //             (0, 0, 0, 0)
            //         } else {
            //             (x, y, w, h)
            //         }
            //     },
            //     //Err(_) => continue,
            //     Err(_) => (0, 0, 0, 0),
            // };

            // // Use empty string for windows with invalid names
            // let name = match self.win_name(win) {
            //     Ok(name) => name,
            //     Err(_) => "".to_owned(),
            // };

            // // Filter out windows that are INPUT_ONLY
            // let (class, state) = self.win_attributes(win)?;
            // // if class == WindowClass::INPUT_ONLY {
            // //     continue;
            // // }

            // windows.push((win, name, typ, class, state, (x as u32, y as u32, w as u32, h as u32)));
            windows.push(win);
        }

        Ok(windows)
    }

    // Helper method to print out the data type
    // println!("DataType NET: {:?}", AtomEnum::from(reply.type_ as u8));
    #[allow(dead_code)]
    pub(crate) fn print_data_type(reply: &GetPropertyReply) {
        println!("DataType: {:?}", AtomEnum::from(reply.type_ as u8));
    }
}
