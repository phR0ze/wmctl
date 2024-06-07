// ## References
// * https://specifications.freedesktop.org/wm-spec/latest
// * https://github.com/psychon/x11rb/blob/master/x11rb/examples/tutorial.rs
// * [ICCCM specification](https://x.org/releases/X11R7.6/doc/xorg-docs/specs/ICCCM/icccm.html)
// * https://www.x.org/wiki/guide/xlib-and-xcb/
//
// ## Details
// * Atoms are unique names that clients can use to communicate information to each other.
// * The window manager is a client of the X server and can be communicated with like any other client.
// * Root window refers to the Window Manager. Sending messages (i.e. SendEvent) to the root window
//   is effectively communicating with the window manager.
//
// ## ICCCM (Inter-Client Communication Conventions Manual)
// * EWMH (Extended Window Manager Hints) specification builds on top of ICCCM and requires that
//   implementors of EWMH also implement ICCCM.
//
// ### ICCCM Hints
// * WM_CLASS           - window name
// * WM_CLIENT_MACHINE  - hostname of the machine
// * WM_ICON_NAME       - icon name
// * WM_NAME            - window name
//
// ### Primitive Functions
// * GetAtomName - get the name of an atom
//
use crate::{atoms::*, model::*, WmCtlError, WmCtlResult};
use std::{collections::HashMap, str};
use tracing::debug;

use x11rb::{
    connection::Connection,
    protocol::xproto::{ConnectionExt as _, *},
    rust_connection::RustConnection,
};

/// Window Manager provides a higher level interface to the underlying EWHM compatible window manager
pub(crate) struct WinMgr {
    conn: RustConnection,            // x11 connection
    atoms: AtomCollection,           // atom cache
    supported: HashMap<u32, String>, // cache of {id => name} for supported functions
    id: u32,                         // window manager id
    name: String,                    // window manager name
    screen: usize,                   // screen number
    root: u32,                       // root window id
    width: u32,                      // screen width
    height: u32,                     // screen height
    desktops: u32,                   // number of desktops
    compositing: bool,               // compositing manager running

    // Crate properties
    pub(crate) work_width: u32,  // work area width (i.e. minus panels)
    pub(crate) work_height: u32, // work areas height (i.e. minus panels)
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
        debug!("connect: initializing connection...");
        let (conn, screen) = x11rb::connect(None)?;

        // Get the screen size
        let (width, height, root) = {
            let screen = &conn.setup().roots[screen];
            (screen.width_in_pixels as u32, screen.height_in_pixels as u32, screen.root)
        };

        // Populate the atoms collection cache
        let atoms = AtomCollection::new(&conn)?.reply()?;

        // Create the window manager object
        let mut wm = WinMgr {
            id: Default::default(),
            name: Default::default(),
            conn,
            atoms,
            supported: Default::default(),
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
        wm.supported = wm.supported()?;

        debug!("connect: screen: {}, root: {}, w: {}, h: {}", screen, root, width, height);
        Ok(wm)
    }

    /// Convert the given Atom id into an Atom name
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.atom_name(1234).unwrap()
    /// ```
    #[allow(dead_code)]
    pub(crate) fn atom_name(&self, id: u32) -> WmCtlResult<String> {
        let reply = self.conn.get_atom_name(id)?.reply()?;
        if let Ok(value) = str::from_utf8(&reply.name) {
            debug!("atom_name: id: {}, name: {}", id, value.to_owned());
            return Ok(value.to_owned());
        }
        return Ok("".to_string());
    }

    /// Convert the given Atom ids into Atom map of id => name. By doing this in bulk
    /// it is far more efficient and faster than calling `atom_name` for each.
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.atom_map([1, 2, 3]).unwrap()
    /// ```
    #[allow(dead_code)]
    pub(crate) fn atom_map(&self, ids: &[u32]) -> WmCtlResult<HashMap<u32, String>> {
        let mut atoms = HashMap::<u32, String>::new();

        // Faster and more efficient to send all requests before calling reply()
        let cookies = ids.iter().map(|id| self.conn.get_atom_name(*id)).collect::<Vec<_>>();

        // Now take the cookies and ids and process the replies
        for (cookie, id) in cookies.into_iter().zip(ids.iter()) {
            let reply = cookie?.reply()?;
            if let Ok(name) = str::from_utf8(&reply.name) {
                atoms.insert(*id, name.to_owned());
                debug!("atom_names: id: {}, name: {}", id, name);
            }
        }
        return Ok(atoms);
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
            supported: self.supported.clone(),
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

    /// Get the Window Manager's supported functions.
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let supported = wm.supported();
    /// ```
    pub(crate) fn supported(&self) -> WmCtlResult<HashMap<u32, String>> {
        let reply = self
            .conn
            .get_property(false, self.root, self.atoms._NET_SUPPORTED, AtomEnum::ATOM, 0, u32::MAX)?
            .reply()?;
        let ids =
            reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_SUPPORTED".to_owned()))?.collect::<Vec<_>>();
        self.atom_map(&ids)
    }

    /// Determine if the given function is supported by the window manager. This will check the
    /// cached set of Window Manager's supported functions for a match.
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
    pub(crate) fn is_supported(&self, atom: u32) -> bool {
        self.supported.get(&atom).is_some()
    }

    /// Get windows optionally all
    /// * when all is true for some reason the window state is not correctly returned
    /// * when all is true the parent window is the root window for all windows
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
        Ok(if all {
            // All windows in the X11 system
            self.conn.query_tree(self.root)?.reply()?.children
        } else {
            // Window manager client windows which is a subset of all windows that have been
            // reparented i.e. new ids and don't map to the same ids as their all windows selves.
            let reply = self
                .conn
                .get_property(false, self.root, self.atoms._NET_CLIENT_LIST, AtomEnum::WINDOW, 0, u32::MAX)?
                .reply()?;
            let children = reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_CLIENT_LIST".to_owned()))?;
            children.collect::<Vec<_>>()
        })
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
    pub(crate) fn window_kind(&self, id: u32) -> WmCtlResult<Kind> {
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
        let _kind = Kind::from(&self.atoms, typ)?;
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
    pub(crate) fn window_state(&self, id: u32) -> WmCtlResult<Vec<State>> {
        // Defined as: _NET_WM_STATE, ATOM[]
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_STATE`
        // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be a single value.
        let reply =
            self.conn.get_property(false, id, self.atoms._NET_WM_STATE, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;

        let mut states = vec![];
        if reply.value_len > 0 {
            for state in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_WM_STATE".to_owned()))? {
                let state = State::from(&self.atoms, state)?;
                states.push(state);
            }
            debug!("win_state: id: {}, state: {:?}", id, states);
        }
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
    /// * Returns non zero based desktop number
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
        let mut desktop = reply.value32().and_then(|mut x| x.next()).map_or(-1, |x| x as i32);

        // Offset to align with how desktops are typically numbered
        if desktop != -1 {
            desktop += 1;
        }

        debug!("win_desktop: id: {}, desktop: {}", id, desktop);
        Ok(desktop as i32)
    }

    /// Get window visual geometry.
    /// Geometry is a calculated value that represents the window's size and position including it's
    /// frame or visually perceived frame. Be careful in calculating from this value as frame/application
    /// borders are added and subtracted and positioning changed in different uses cases called out
    /// below to make these values more intuitive visually. Other apps like xdotool or xwininfo use
    /// the --frame option to include the window manager's frame in the calculation which is somewhat
    /// akin to what is happending here only this also takes into account Client Side Decorations (CSD).
    ///
    /// * For Window Manager decorated windows this means this function is computing the window size
    ///   plus window manager's border decoration as this gives an intuitively understandable visual
    ///   window size on the screen. Positioning is also adjusted in this case to subtract the borders
    ///   for a total visual space on screen experience.
    /// * For Client Side Decorated (CSD) windows this means window size minus CSD borders as CSD windows
    ///   e.g. GTK apps have a semi-transparent 23,23,15,31 border that is reported as part of the
    ///   window's total size but isn't visible and thus is being subtracted in this function to return
    ///   only an intuitively understandable visual window size on the screen. Positioning was also
    ///   adjusted in this case to add the borders thus ignoring the CSD borders from a visual on screen
    ///   perspective.
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
    pub(crate) fn window_visual_geometry(&self, id: u32) -> WmCtlResult<(i32, i32, u32, u32)> {
        let (mut x, mut y, mut w, mut h) = self.window_geometry(id)?;

        // Account for CSD borders
        let mut is_gtk = false;
        if let Ok((l, r, t, b)) = self.window_gtk_borders(id) {
            if l > 0 || r > 0 || t > 0 || b > 0 {
                w = w - l - r;
                h = h - t - b;
                x = x + l as i32;
                y = y + t as i32;
                is_gtk = true;
            }
        }
        if !is_gtk {
            if let Ok((l, r, t, b)) = self.window_borders(id) {
                w = w + l + r;
                h = h + t + b;
                x = x - l as i32;
                y = y - t as i32;
            }
        }

        debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", id, x, y, w, h);
        Ok((x, y, w, h))
    }

    /// Get window geometry as reported by the window manager without any adjustments.
    /// * This is the window's position and size without any window manager decorations included
    ///   which can be counter intuitive visually if you're trying to understand the window's
    ///   position and size on the screen.
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let (x, y, w, h) = wm.window_raw_geometry(1234).unwrap()
    /// ```
    pub(crate) fn window_geometry(&self, id: u32) -> WmCtlResult<(i32, i32, u32, u32)> {
        // References
        // * https://github.com/psychon/x11rb/blob/c55337f839fd03eeb77996b776a736fcf8136dd9/x11rb/examples/tutorial.rs#L1840
        //
        // The returned x, y location is relative to the upper-left corner of its parent window
        // making the values completely useless as it doesn't relate to what we see on the screen
        // in an intuitive way. In order to overcome this problem, we need to take a two-step approach.
        // First, we find out the Id of the parent window, which might not be the root window. We then
        // translate the relative coordinates in relation to the parent window to get actual screen
        // coordinates that have real world meaning relative to 0, 0 of the screen.
        let g = self.conn.get_geometry(id)?.reply()?;
        let (w, h) = (g.width as u32, g.height as u32);

        let mut parent = self.window_parent(id)?.id;
        let (x, y) = if parent != self.root {
            // NOTE: Despite the XCB directions to use the window's parent for the relative translation
            // I've found in XFWM that this doesn't report the window's position correctly unless we
            // always use the root window for the relative base to translate from.
            parent = self.root;

            // NOTE: Tests show that get_geometry returns the window's (x, y) coordinates with an offset
            // taking into account the left and top borders of the window. For example a window that
            // visually appears to be at (0, 0) on the screen but has borders of (4, 4, 28, 4) will have
            // an (x, y) of (4, 28) returned from get_geometry. Additionally tests show that translate
            // adds the same left and top border values. Thus as with [wmctrl] and other projects that
            // input the get_geometry (x, y) values into translate function as is commonly shown in many
            // tutorials the resulting (x, y) values are being incorrectly reported as (8, 56) instead of
            // just (4, 28) as they are incorrectly geting double the left and right borders.
            // Thus as done in the xwininfo project simply using (0, 0) as the input parameters for
            // translate's (x, y) will correctly add only one set of borders to the (x, y) values.
            let tx = self.conn.translate_coordinates(id, parent, 0, 0)?.reply()?;
            (tx.dst_x as i32, tx.dst_y as i32)
        } else {
            // If parent is the root window then the x, y values are already correct
            (g.x as i32, g.y as i32)
        };

        debug!("win_raw_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", id, x, y, w, h);
        Ok((x, y, w, h))
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
        // Window managers decorate windows with boarders and title bars. The _NET_FRAME_EXTENTS
        // defined as: left, right, top, bottom, CARDINAL[4]/32 will retrieve these values via
        // `get_property` api call with the use of the `self.atoms._NET_FRAME_EXTENTS`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the
        // `reply.value32()`.
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

    /// Determine if this window is a GTK application
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let win = window(12345);
    /// let result = win.window_is_gtk();
    /// ```
    pub(crate) fn window_is_gtk(&self, id: u32) -> bool {
        if let Ok((l, r, t, b)) = self.window_gtk_borders(id) {
            if l > 0 || r > 0 || t | 0 | b > 0 {
                return true;
            }
        }
        false
    }

    /// Get GNOME window borders
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// let win = window(12345);
    /// let (l, r, t, b) = wm.window_gnome_borders().unwrap();
    /// ```
    #[allow(dead_code)]
    pub(crate) fn window_gtk_borders(&self, id: u32) -> WmCtlResult<(u32, u32, u32, u32)> {
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
        let reply = self
            .conn
            .get_property(false, id, self.atoms._GTK_FRAME_EXTENTS, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;

        // Don't abort if the property is not found as its not required
        if reply.value.is_empty() {
            return Ok((0, 0, 0, 0));
        }

        let mut values = reply.value32().ok_or(WmCtlError::PropertyNotFound("_GTK_FRAME_EXTENTS".to_owned()))?;
        let l = values.next().ok_or(WmCtlError::PropertyNotFound("_GTK_FRAME_EXTENTS left".to_owned()))?;
        let r = values.next().ok_or(WmCtlError::PropertyNotFound("_GTK_FRAME_EXTENTS right".to_owned()))?;
        let t = values.next().ok_or(WmCtlError::PropertyNotFound("_GTK_FRAME_EXTENTS top".to_owned()))?;
        let b = values.next().ok_or(WmCtlError::PropertyNotFound("_GTK_FRAME_EXTENTS bottom".to_owned()))?;

        debug!("win_gnome_borders: id: {}, l: {}, r: {}, t: {}, b: {}", id, l, r, t, b);
        Ok((l, r, t, b))
    }

    /// Get all properties for the given window as a sorted list
    ///
    /// ### Arguments
    /// * `id` - id of the window to pull properteries for
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.window_properties(1234).unwrap();
    /// ```
    pub(crate) fn window_properties(&self, id: u32) -> WmCtlResult<Vec<crate::Property>> {
        let reply = self.conn.list_properties(id)?.reply()?;

        // Get atoms names
        let atom_map = self.atom_map(&reply.atoms)?;

        // Create properties from the atoms and sort by name
        let mut props = atom_map.iter().map(|x| crate::Property::new(*x.0, x.1)).collect::<Vec<_>>();
        props.sort_by(|a, b| a.name.cmp(&b.name));
        // for prop in props.iter() {
        //     let reply = self.conn.get_property(false, id, prop.id, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        // }
        Ok(props)
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
    pub(crate) fn window_attributes(&self, id: u32) -> WmCtlResult<crate::MapState> {
        let attr = self.conn.get_window_attributes(id)?.reply()?;
        debug!(
            "win_attributes: id: {}, win_gravity: {:?}, bit_gravity: {:?}",
            id, attr.win_gravity, attr.bit_gravity
        );
        //Ok((Class::from(attr.class.into())?, crate::MapState::from(attr.map_state.into())?))
        Ok(crate::MapState::from(attr.map_state.into())?)
    }

    /// Map the window on the screen
    ///
    /// ### Arguments
    /// * `id` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let wm = WinMgr::connect().unwrap();
    /// wm.map_window().unwrap();
    /// ```
    pub(crate) fn map_window(&self, id: u32) -> WmCtlResult<()> {
        debug!("map_window: id: {}", id);
        self.conn.map_window(id)?;
        Ok(())
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
        &self, id: u32, gravity: Option<u32>, x: Option<i32>, y: Option<i32>, w: Option<u32>, h: Option<u32>,
    ) -> WmCtlResult<()> {
        self.conn.configure_window(id, &ConfigureWindowAux::new().x(x).y(y).width(w).height(h))?;
        self.conn.flush()?; // Requires the flush to work

        // Old implementation below doesn't allow for negative (x, y) coordinates
        // ----------------------------------------------------------------
        // // Construct the move resize message
        // // Gravity is defined as the lower byte of the move resize flags 32bit value
        // // https://tronche.com/gui/x/xlib/window/attributes/gravity.html
        // // Defines how the window will shift as it grows or shrinks during a shape change operation.
        // // The default value is NorthWest which means that the window will grow to the right and down
        // // and will shrink up and left. By changing this to center you can get a more distributed growth
        // // or shrink perception.
        // let mut flags = gravity.unwrap_or(0);

        // // Define the second byte of the move resize flags 32bit value
        // // Used to indicate that the associated value has been changed and needs to be acted upon
        // if x.is_some() {
        //     flags |= MOVE_RESIZE_WINDOW_X;
        // }
        // if y.is_some() {
        //     flags |= MOVE_RESIZE_WINDOW_Y;
        // }
        // if w.is_some() {
        //     flags |= MOVE_RESIZE_WINDOW_WIDTH;
        // }
        // if h.is_some() {
        //     flags |= MOVE_RESIZE_WINDOW_HEIGHT;
        // }

        // self.send_event(ClientMessageEvent::new(
        //     32,
        //     id,
        //     self.atoms._NET_MOVERESIZE_WINDOW,
        //     [flags, x.unwrap_or(0), y.unwrap_or(0), w.unwrap_or(0), h.unwrap_or(0)],
        // ))?;

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
