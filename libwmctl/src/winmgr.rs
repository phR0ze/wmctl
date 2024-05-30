use crate::{atoms::*, model::*, WmCtlError, WmCtlResult};
use std::{collections::HashMap, str};
use tracing::{debug, trace};

use x11rb::{
    connection::Connection,
    protocol::xproto::{self, ConnectionExt as _, *},
    rust_connection::RustConnection,
};

/// Window Manager provides a higher level interface to the underlying EWHM compatible window manager
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
    /// let wm = WinMgr::connect().unwrap();
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
    /// let wm = WinMgr::connect().unwrap();
    /// wm.info().unwrap()
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
    /// let wm = WinMgr::connect().unwrap();
    /// wm.active_window().unwrap()
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

    /// Determine if the given function is supported by the window manager
    ///
    /// ### Arguments
    /// * `atom` - atom to lookup to see if its supported
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.supported(wm.atoms._NET_MOVERESIZE_WINDOW);
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
    /// let wm = WinMgr::connect().unwrap();
    /// wm.windows(false).unwrap()
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

    /// Get window pid
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.window_pid(1234)
    /// ```
    pub(crate) fn window_pid(&self, id: u32) -> WmCtlResult<i32> {
        // Defined as: _NET_WM_PID, CARDINAL/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_PID`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be a single value.
        let reply =
            self.conn.get_property(false, id, self.atoms._NET_WM_PID, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let pid = reply
            .value32()
            .and_then(|mut x| x.next())
            .ok_or(WmCtlError::PropertyNotFound("_NET_WM_PID".to_owned()))?;
        debug!("win_pid: id: {}, pid: {:?}", id, pid);
        Ok(pid as i32)
    }

    /// Get window name
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.window_name(1234)
    /// ```
    pub(crate) fn window_name(&self, id: u32) -> WmCtlResult<String> {
        // Defined as: _NET_WM_NAME, UTF8_STRING
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_NAME`
        // request message with a `AtomEnum::UTF8_STRING` type response and we can use the `reply.value` accessor to
        // retrieve the value.

        // First try the _NET_WM_VISIBLE_NAME
        let reply = self
            .conn
            .get_property(false, id, self.atoms._NET_WM_VISIBLE_NAME, self.atoms.UTF8_STRING, 0, u32::MAX)?
            .reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using _NET_WM_VISIBLE_NAME for: {}", value);
                    return Ok(value.to_owned());
                }
            }
        }

        // Next try the _NET_WM_NAME
        let reply = self
            .conn
            .get_property(false, id, self.atoms._NET_WM_NAME, self.atoms.UTF8_STRING, 0, u32::MAX)?
            .reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using _NET_WM_NAME for: {}", value);
                    return Ok(value.to_owned());
                }
            }
        }

        // Fall back on the WM_NAME
        let reply =
            self.conn.get_property(false, id, AtomEnum::WM_NAME, AtomEnum::STRING, 0, u32::MAX)?.reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using WM_NAME for: {}", value);
                    return Ok(value.to_owned());
                }
            }
        }

        // No valid name was found
        Err(WmCtlError::PropertyNotFound("_NET_WM_NAME | _WM_NAME".to_owned()).into())
    }

    /// Get window class which is typically the the application's name
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.window_class(1234)
    /// ```
    pub(crate) fn window_class(&self, id: u32) -> WmCtlResult<String> {
        let reply =
            self.conn.get_property(false, id, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, u32::MAX)?.reply()?;

        // Skip the first null terminated string and extract the second
        let iter = reply.value.into_iter().skip_while(|x| *x != 0).skip(1).take_while(|x| *x != 0);

        // Extract the second null terminated string
        let class = str::from_utf8(&iter.collect::<Vec<_>>())?.to_owned();
        debug!("win_class: id: {}, class: {}", id, class);
        Ok(class)
    }

    /// Get window kind
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.window_kind(1234)
    /// ```
    pub(crate) fn window_kind(&self, id: u32) -> WmCtlResult<WinKind> {
        // Defined as: _NET_WM_WINDOW_TYPE, ATOM[]/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_WINDOW_TYPE`
        // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
        // retrieve the value.
        let reply = self
            .conn
            .get_property(false, id, self.atoms._NET_WM_WINDOW_TYPE, AtomEnum::ATOM, 0, u32::MAX)?
            .reply()?;
        let typ = reply
            .value32()
            .and_then(|mut x| x.next())
            .ok_or(WmCtlError::PropertyNotFound("_NET_WM_WINDOW_TYPE".to_owned()))?;
        let _kind = WinKind::from(&self.atoms, typ)?;
        debug!("win_kind: id: {}, kind: {:?}", id, _kind);
        Ok(_kind)
    }

    /// Get window state
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.window_state(1234)
    /// ```
    pub(crate) fn window_state(&self, id: u32) -> WmCtlResult<Vec<WinState>> {
        // Defined as: _NET_WM_STATE, ATOM[]
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_STATE`
        // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be a single value.
        let reply =
            self.conn.get_property(false, id, self.atoms._NET_WM_STATE, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;

        let mut states = vec![];
        for state in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_WM_STATE".to_owned()))? {
            let state = WinState::from(&self.atoms, state)?;
            states.push(state);
        }
        debug!("win_state: id: {}, state: {:?}", id, states);
        Ok(states)
    }

    /// Get window parent
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.window_parent(1234)
    /// ```
    #[allow(dead_code)]
    pub(crate) fn window_parent(&self, id: u32) -> WmCtlResult<crate::Window> {
        let tree = self.conn.query_tree(id)?.reply()?;
        let parent_id = tree.parent;
        debug!("win_parent: id: {}, parent: {:?}", id, parent_id);
        Ok(crate::Window::new(parent_id))
    }

    /// Get window desktop
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.window_desktop(1234)
    /// ```
    pub(crate) fn window_desktop(&self, id: u32) -> WmCtlResult<i32> {
        // Defined as: _NET_WM_DESKTOP desktop, CARDINAL/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_DESKTOP`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be a single value.
        let reply = self
            .conn
            .get_property(false, id, self.atoms._NET_WM_DESKTOP, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;
        let desktop = reply
            .value32()
            .and_then(|mut x| x.next())
            .ok_or(WmCtlError::PropertyNotFound("_NET_WM_DESKTOP".to_owned()))?;
        debug!("win_desktop: id: {}, desktop: {}", id, desktop);
        Ok(desktop as i32)
    }

    /// Get window geometry
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let (x, y, w, h) = wm.window_geometry(1234).unwrap()
    /// ```
    pub(crate) fn window_geometry(&self, id: u32) -> WmCtlResult<(i32, i32, u32, u32)> {
        // The returned x, y location is relative to its parent window making the values completely
        // useless. However using `translate_coordinates` we can have the window manager map those
        // useless values into real world cordinates by passing it the root as the relative window.

        // Get width and heith and useless relative location values
        let g = self.conn.get_geometry(id)?.reply()?;

        // Translate the useless retative location values to to real world values
        let t = self.conn.translate_coordinates(id, self.root, g.x, g.y)?.reply()?;

        let (x, y, w, h) = (t.dst_x, t.dst_y, g.width, g.height);
        debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", id, x, y, w, h);
        Ok((x as i32, y as i32, w as u32, h as u32))
    }

    /// Get window frame border values added by the window manager
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let win = window(12345);
    /// let (l, r, t, b) = wm.window_borders().unwrap();
    /// ```
    pub(crate) fn window_borders(&self, id: u32) -> WmCtlResult<(u32, u32, u32, u32)> {
        // Window managers (a.k.a server-side) decorate windows with boarders and title bars. The
        // _NET_FRAME_EXTENTS defined as: left, right, top, bottom, CARDINAL[4]/32 will retrieve
        // these values via `get_property` api call with the use of the `self.atoms._NET_FRAME_EXTENTS`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the
        // `reply.value32()`. Client-side decorations (CSD) is where the application draws the
        // window decorations (borders, titlebar etc...) itself. In the CSD architecture used by GNOME
        // the application draws decorations including the shadows. The shadows are click-through and
        // semitransparent, but they are still part of the app window. To account for this the GNOME
        // app will set the _GTK_FRAME_EXTENTS property showing the space consumed by these shadows that
        // can be effectively used as the window borders rather than the window manager borders provided
        // by _NET_FRAME_EXTENTS. _GTK_FRAME_EXTENTS is defined as: left, right, top, bottom
        //
        // This is why `wmctl list` will show evince has geometry of 1280x1415 and borders of 0, 0, 0, 0
        // while `xprop -id 104857608 | grep EXTENT` shows `_GTK_FRAME_EXTENTS(CARDINAL) = 23, 23, 15, 31`
        // which would mean that the window geometry is actually 1280-23-23x1415-15-31 or 1234x1369.
        let reply = self
            .conn
            .get_property(false, id, self.atoms._NET_FRAME_EXTENTS, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;
        let mut values = reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS".to_owned()))?;
        let l = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS left".to_owned()))?;
        let r = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS right".to_owned()))?;
        let t = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS top".to_owned()))?;
        let b = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS bottom".to_owned()))?;
        debug!("win_borders: id: {}, l: {}, r: {}, t: {}, b: {}", id, l, r, t, b);
        Ok((l, r, t, b))
    }

    /// Get all properties for the given window
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.active_win().unwrap();
    /// ```
    pub(crate) fn window_properties(&self, id: u32) -> WmCtlResult<u32> {
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

    /// Get window attribrtes
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let (class, state) = wm.win_attributes(12345).unwrap();
    /// ```
    #[allow(dead_code)]
    pub(crate) fn window_attributes(&self, id: u32) -> WmCtlResult<(WinClass, WinMap)> {
        let attr = self.conn.get_window_attributes(id)?.reply()?;
        debug!(
            "win_attributes: id: {}, win_gravity: {:?}, bit_gravity: {:?}",
            id, attr.win_gravity, attr.bit_gravity
        );
        Ok((WinClass::from(attr.class.into())?, WinMap::from(attr.map_state.into())?))
    }

    /// Maximize the window both horizontally and vertically
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.maximize_window().unwrap();
    /// ```
    pub(crate) fn maximize_window(&self, id: u32) -> WmCtlResult<()> {
        self.send_event(ClientMessageEvent::new(
            32,
            id,
            self.atoms._NET_WM_STATE,
            [
                WINDOW_STATE_ACTION_ADD,
                self.atoms._NET_WM_STATE_MAXIMIZED_HORZ,
                self.atoms._NET_WM_STATE_MAXIMIZED_VERT,
                0,
                0,
            ],
        ))?;
        debug!("maximize: id: {}", id);
        Ok(())
    }

    /// Move and resize window
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Arguments
    /// * `gravity` - gravity to use when resizing the window, defaults to NorthWest
    /// * `x` - x coordinate to use for the window during positioning
    /// * `y` - y coordinate to use for the window during positioning
    /// * `w` - width to resize the window to
    /// * `h` - height to resize the window to
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let win = window(12345);
    /// win.move_resize_win(None, Some(0), Some(0), Some(500), Some(500)).unwrap();
    /// ```
    pub(crate) fn move_resize_window(
        &self, id: u32, gravity: Option<u32>, x: Option<u32>, y: Option<u32>, w: Option<u32>, h: Option<u32>,
    ) -> WmCtlResult<()> {
        // Construct the move resize message
        // Gravity is defined as the lower byte of the move resize flags 32bit value
        // https://tronche.com/gui/x/xlib/window/attributes/gravity.html
        // Defines how the window will shift as it grows or shrinks during a shape change operation.
        // The default value is NorthWest which means that the window will grow to the right and down
        // and will shrink up and left. By changing this to center you can get a more distributed growth
        // or shrink perception.
        let mut flags = gravity.unwrap_or(0);

        // Define the second byte of the move resize flags 32bit value
        // Used to indicate that the associated value has been changed and needs to be acted upon
        if x.is_some() {
            flags |= MOVE_RESIZE_WINDOW_X;
        }
        if y.is_some() {
            flags |= MOVE_RESIZE_WINDOW_Y;
        }
        if w.is_some() {
            flags |= MOVE_RESIZE_WINDOW_WIDTH;
        }
        if h.is_some() {
            flags |= MOVE_RESIZE_WINDOW_HEIGHT;
        }

        self.send_event(ClientMessageEvent::new(
            32,
            id,
            self.atoms._NET_MOVERESIZE_WINDOW,
            [flags, x.unwrap_or(0), y.unwrap_or(0), w.unwrap_or(0), h.unwrap_or(0)],
        ))?;

        debug!("move_resize: id: {}, g: {:?}, x: {:?}, y: {:?}, w: {:?}, h: {:?}", id, gravity, x, y, w, h);
        Ok(())
    }

    /// Remove the MaxVert and MaxHorz states
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.unmaximize_window().unwrap();
    /// ```
    pub(crate) fn unmaximize_window(&self, id: u32) -> WmCtlResult<()> {
        self.send_event(ClientMessageEvent::new(
            32,
            id,
            self.atoms._NET_WM_STATE,
            [
                WINDOW_STATE_ACTION_REMOVE,
                self.atoms._NET_WM_STATE_MAXIMIZED_HORZ,
                self.atoms._NET_WM_STATE_MAXIMIZED_VERT,
                0,
                0,
            ],
        ))?;
        debug!("unmaximize: id: {}", id);
        Ok(())
    }

    /// Get window manager's window id and name
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let (id, name) = wm.winmgr().unwrap();
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
        let name = self.window_name(id)?;
        debug!("winmgr: id: {}, name: {}", id, name);
        Ok((id, name))
    }

    /// Get desktop work area
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let (w, h) = wm.workarea().unwrap();
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
    /// let wm = WinMgr::connect().unwrap();
    /// wm.compositing().unwrap();
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
    /// let wm = WinMgr::connect().unwrap();
    /// wm.desktops().unwrap();
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

    /// Send the event ensuring that a flush is called and that the message was precisely
    /// executed in the case of a resize/move.
    ///
    /// ### Arguments
    /// * `msg` - the client message event to send
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let flags = MOVE_RESIZE_WINDOW_WIDTH | MOVE_RESIZE_WINDOW_HEIGHT;
    /// wm.send_event(ClientMessageEvent::new(32, win, wm.atoms._NET_MOVERESIZE_WINDOW,
    ///     [flags, 0, 0, 500, 500])).unwrap();
    /// ```
    fn send_event(&self, msg: ClientMessageEvent) -> WmCtlResult<()> {
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

    // Helper method to print out the data type
    // println!("DataType NET: {:?}", AtomEnum::from(reply.type_ as u8));
    #[allow(dead_code)]
    fn print_data_type(reply: &GetPropertyReply) {
        println!("DataType: {:?}", AtomEnum::from(reply.type_ as u8));
    }
}
