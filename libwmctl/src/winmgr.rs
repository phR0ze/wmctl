//! `WmCtl` implements the [Extended Window Manager Hints (EWMH) specification](https://specifications.freedesktop.org/wm-spec/latest/)
//! as a way to integrate with EWMH compatible window managers. The EWHM spec builds on the lower
//! level Inter Client Communication Conventions Manual (ICCCM) to define interactions between
//! window managers, compositing managers and applications.
//!
//! [Root Window Properties](https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html)  
//! The EWMH spec defines a number of properties that EWHM compliant window managers will maintain
//! and return to clients requesting information. `WmCtl` taps into the message queue to retrieve
//! details about a given window and to than manipulate the given window as desired.
//!
//! `wmctl` uses `WmCtl` with pre-defined shapes and positions to manipulate how a window should
//! be shaped and positioned on the screen in an ergonomic way; however `WmCtl` could be used
//! for a variety of reasons.
use crate::{atoms::*, model::*, WmCtlError, WmCtlResult};
use std::{collections::HashMap, str, sync::Arc};
use tracing::{debug, trace};

use x11rb::{
    connection::Connection,
    protocol::xproto::{self, ConnectionExt as _, *},
    rust_connection::RustConnection,
};

/// Window Manager control implements the EWMH protocol using x11rb to provide a simplified access
/// layer to EWHM compatible window managers.
pub(crate) struct WinMgr {
    pub(crate) conn: RustConnection, // x11 connection
    pub atoms: AtomCollection,       // atom cache
    supported: HashMap<u32, bool>,   // cache for supported functions
    pub(crate) id: u32,              // window manager id
    pub(crate) name: String,         // window manager name
    pub(crate) screen: usize,        // screen number
    pub(crate) root: u32,            // root window id
    pub(crate) width: u32,           // screen width
    pub(crate) height: u32,          // screen height
    pub(crate) work_width: u32,      // work area width (i.e. minus panels)
    pub(crate) work_height: u32,     // work areas height (i.e. minus panels)
    pub(crate) desktops: u32,        // number of desktops
    pub(crate) compositing: bool,    // compositing manager running
}

impl WinMgr {
    /// Create the window manager control instance and connect to the X11 server
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// ```
    pub(crate) fn connect() -> WmCtlResult<Self> {
        let (conn, screen) = x11rb::connect(None)?;

        // Get the screen size
        let (width, height, root) = {
            let screen = &conn.setup().roots[screen];
            (screen.width_in_pixels as u32, screen.height_in_pixels as u32, screen.root)
        };

        // Populate the supported functions cache
        let (atoms, supported) = WinMgr::init_caching(&conn, root)?;

        // Create the window manager object
        let mut wm = WinMgr {
            id: Default::default(),
            name: Default::default(),
            conn,
            atoms,
            supported,
            screen,
            root,
            width,
            height,
            work_width: Default::default(),
            work_height: Default::default(),
            desktops: Default::default(),
            compositing: Default::default(),
        };

        // Fill in missing properties that require a connection and supported atoms init_caching
        let (id, name) = wm.id()?;
        wm.id = id;
        wm.name = name;
        let (width, height) = wm.workarea()?;
        wm.work_width = width as u32;
        wm.work_height = height as u32;
        wm.desktops = wm.desktops()?;
        wm.compositing = wm.compositing()?;

        debug!("connect: screen: {}, root: {}, w: {}, h: {}", screen, root, width, height);
        Ok(wm)
    }

    /// Get window manager's informational properties
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// let (id, name) = wmctl.winmgr().unwrap();
    /// ```
    pub(crate) fn info(&self) -> WmCtlResult<Info> {
        Ok(Info {
            id: self.id,
            name: self.name.clone(),
            root_win_id: self.root,
            work_area: (self.work_width, self.work_height),
            screen_size: (self.width, self.height),
            desktops: self.desktops,
            compositing: self.compositing,
        })
    }

    /// Get the active window id
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// wmctl.active_win().unwrap();
    /// ```
    pub(crate) fn active_window(&self) -> WmCtlResult<u32> {
        // Defined as: _NET_ACTIVE_WINDOW, WINDOW/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_ACTIVE_WINDOW`
        // request message with a `AtomEnum::WINDOW` type response and we can use the `reply.value32()` accessor to
        // retrieve the value.
        let reply = self
            .conn
            .get_property(false, self.root, self.atoms._NET_ACTIVE_WINDOW, AtomEnum::WINDOW, 0, u32::MAX)?
            .reply()?;
        let win = reply
            .value32()
            .and_then(|mut x| x.next())
            .ok_or(WmCtlError::PropertyNotFound("_NET_ACTIVE_WINDOW".to_owned()))?;
        debug!("active_win: {}", win);
        Ok(win)
    }

    /// Send the event ensuring that a flush is called and that the message was precisely
    /// executed in the case of a resize/move.
    ///
    /// ### Arguments
    /// * `msg` - the client message event to send
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// let flags = MOVE_RESIZE_WINDOW_WIDTH | MOVE_RESIZE_WINDOW_HEIGHT;
    /// wmctl.send_event(ClientMessageEvent::new(32, win, wmctl.atoms._NET_MOVERESIZE_WINDOW,
    ///     [flags, 0, 0, 500, 500])).unwrap();
    /// ```
    pub(crate) fn send_event(&self, msg: ClientMessageEvent) -> WmCtlResult<()> {
        let mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
        self.conn.send_event(false, self.root, mask, &msg)?.check()?;
        self.conn.flush()?;
        debug!("send_event: win: {}", msg.window);

        // I've found that Xfwm4 does not precisely resize a window on the first request. It may be
        // this is a function of decorating the window during a redraw. At any rate because of this
        // unfortunate shortcoming we have to send the event a second time.
        if msg.type_ == self.atoms._NET_MOVERESIZE_WINDOW {
            std::thread::sleep(std::time::Duration::from_millis(50));
            self.conn.send_event(false, self.root, mask, &msg)?.check()?;
            self.conn.flush()?;
            debug!("send_event: win: {}", msg.window);
        }
        Ok(())
    }

    /// Determine if the given function is supported by the window manager
    ///
    /// ### Arguments
    /// * `atom` - atom to lookup to see if its supported
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// wmctl.supported(wmctl.atoms._NET_MOVERESIZE_WINDOW);
    /// ```
    #[allow(dead_code)]
    pub(crate) fn supported(&self, atom: u32) -> bool {
        self.supported.get(&atom).is_some()
    }
    /// Get windows optionally all
    ///
    /// ### Arguments
    /// * `all` - default is to get all windows controlled by the window manager, when all is true get the super set of x11 windows
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// wmctl.windows(false).unwrap();
    /// ```
    pub(crate) fn windows(&self, all: bool) -> WmCtlResult<Vec<u32>> {
        let mut windows = vec![];
        if all {
            // All windows in the X11 system
            let tree = self.conn.query_tree(self.root)?.reply()?;
            for win in tree.children {
                windows.push(win);
            }
        } else {
            // Window manager client windows which is a subset of all windows that have been
            // reparented i.e. new ids and don't map to the same ids as their all windows selves.
            let reply = self
                .conn
                .get_property(false, self.root, self.atoms._NET_CLIENT_LIST, AtomEnum::WINDOW, 0, u32::MAX)?
                .reply()?;
            for win in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_CLIENT_LIST".to_owned()))? {
                windows.push(win)
            }
        }
        Ok(windows)
    }
    /// Get window manager's window id and name
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// let (id, name) = wmctl.winmgr().unwrap();
    /// ```
    fn id(&self) -> WmCtlResult<(u32, String)> {
        let reply = self
            .conn
            .get_property(false, self.root, self.atoms._NET_SUPPORTING_WM_CHECK, AtomEnum::WINDOW, 0, u32::MAX)?
            .reply()?;
        let id = reply
            .value32()
            .and_then(|mut x| x.next())
            .ok_or(WmCtlError::PropertyNotFound("_NET_SUPPORTING_WM_CHECK".to_owned()))?;
        let name = crate::Window::new(id).name()?;
        debug!("winmgr: id: {}, name: {}", id, name);
        Ok((id, name))
    }

    /// Get desktop work area
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// let (w, h) = wmctl.workarea().unwrap();
    /// ```
    fn workarea(&self) -> WmCtlResult<(u16, u16)> {
        // Defined as: _NET_WORKAREA, x, y, width, height CARDINAL[][4]/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WORKAREA`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be 4 for each desktop as defined (x, y, width, height).
        let reply = self
            .conn
            .get_property(false, self.root, self.atoms._NET_WORKAREA, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;
        let mut values = reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA".to_owned()))?;
        let x = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA x".to_owned()))?;
        let y = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA y".to_owned()))?;
        let w = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA width".to_owned()))?;
        let h = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA height".to_owned()))?;
        debug!("work_area: x: {}, y: {}, w: {}, h: {}", x, y, w, h);

        // x and y are always zero so dropping them
        Ok((w as u16, h as u16))
    }

    /// Check if a composit manager is running
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// wmctl.compositing().unwrap();
    /// ```
    fn compositing(&self) -> WmCtlResult<bool> {
        // Defined as: _NET_WM_CM_Sn
        // For each screen the compositing manager manages they MUST acquire ownership of a
        // selection named _NET_WM_CM_Sn, where the suffix `n` is the screen number.
        let atom = format!("_NET_WM_CM_S{}", self.screen);
        let atom = self.conn.intern_atom(false, atom.as_bytes())?.reply()?.atom;
        let reply = self.conn.get_selection_owner(atom)?.reply()?;
        let result = reply.owner != x11rb::NONE;
        debug!("composite_manager: {}", result);
        Ok(result)
    }

    /// Get number of desktops
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// wmctl.desktops().unwrap();
    /// ```
    fn desktops(&self) -> WmCtlResult<u32> {
        // Defined as: _NET_NUMBER_OF_DESKTOPS, CARDINAL/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_NUMBER_OF_DESKTOPS`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the value.
        let reply = self
            .conn
            .get_property(false, self.root, self.atoms._NET_NUMBER_OF_DESKTOPS, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;
        let num = reply
            .value32()
            .and_then(|mut x| x.next())
            .ok_or(WmCtlError::PropertyNotFound("_NET_NUMBER_OF_DESKTOPS".to_owned()))?;
        debug!("desktops: {}", num);
        Ok(num)
    }

    /// Get window attribrtes
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wmctl = WmCtl::connect().unwrap();
    /// let (class, state) = wmctl.win_attributes(12345).unwrap();
    /// ```
    #[allow(dead_code)]
    pub fn win_attributes(&self, win: xproto::Window) -> WmCtlResult<(WinClass, WinMap)> {
        let attr = self.conn.get_window_attributes(win)?.reply()?;
        debug!(
            "win_attributes: id: {}, win_gravity: {:?}, bit_gravity: {:?}",
            win, attr.win_gravity, attr.bit_gravity
        );
        Ok((WinClass::from(attr.class.into())?, WinMap::from(attr.map_state.into())?))
    }

    // Initialize caching
    fn init_caching(conn: &RustConnection, root: u32) -> WmCtlResult<(AtomCollection, HashMap<u32, bool>)> {
        debug!("initializing caching...");

        // Cache atoms
        let atoms = AtomCollection::new(conn)?.reply()?;

        // Cache supported functions
        let mut supported = HashMap::<u32, bool>::new();
        let reply = conn.get_property(false, root, atoms._NET_SUPPORTED, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
        for atom in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_SUPPORTED".to_owned()))? {
            trace!("supported: {}", atom);
            supported.insert(atom, true);
        }
        debug!("caching initialized");
        Ok((atoms, supported))
    }

    // Helper method to print out the data type
    // println!("DataType NET: {:?}", AtomEnum::from(reply.type_ as u8));
    #[allow(dead_code)]
    fn print_data_type(reply: &GetPropertyReply) {
        println!("DataType: {:?}", AtomEnum::from(reply.type_ as u8));
    }
}
