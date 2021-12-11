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
use crate::{WmCtlResult, model::*, WmCtlError, WinClass, WinMap, WinState, WinType};
use std::{str, ops::Deref, collections::HashMap};
use tracing::{trace, debug};

use x11rb::{
    atom_manager,
    connection::Connection,
    protocol::xproto::{ConnectionExt as _, self, *},
    wrapper::ConnectionExt as _,
    rust_connection::RustConnection,
};

// A collection of the atoms we will need.
atom_manager! {
    pub(crate) AtomCollection: AtomCollectionCookie {
        _NET_ACTIVE_WINDOW,
        _NET_CLIENT_LIST,
        _NET_CLIENT_LIST_STACKING,
        _NET_CLOSE_WINDOW,
        _NET_CURRENT_DESKTOP,
        _NET_DESKTOP_GEOMETRY,
        _NET_DESKTOP_LAYOUT,
        _NET_DESKTOP_NAMES,
        _NET_DESKTOP_VIEWPORT,
        _NET_FRAME_EXTENTS,
        _NET_MOVERESIZE_WINDOW,
        _NET_NUMBER_OF_DESKTOPS,
        _NET_REQUEST_FRAME_EXTENTS,
        _NET_SHOWING_DESKTOP,
        _NET_SUPPORTED,
        _NET_SUPPORTING_WM_CHECK,
        _NET_SYSTEM_TRAY_OPCODE,
        _NET_WM_ACTION_ABOVE,
        _NET_WM_ACTION_BELOW,
        _NET_WM_ACTION_CHANGE_DESKTOP,
        _NET_WM_ACTION_CLOSE,
        _NET_WM_ACTION_FULLSCREEN,
        _NET_WM_ACTION_MAXIMIZE_HORZ,
        _NET_WM_ACTION_MAXIMIZE_VERT,
        _NET_WM_ACTION_MINIMIZE,
        _NET_WM_ACTION_MOVE,
        _NET_WM_ACTION_RESIZE,
        _NET_WM_ACTION_SHADE,
        _NET_WM_ACTION_STICK,
        _NET_WM_ALLOWED_ACTIONS,
        _NET_WM_BYPASS_COMPOSITOR,
        _NET_WM_CONTEXT_HELP,
        _NET_WM_DESKTOP,
        _NET_WM_FULLSCREEN_MONITORS,
        _NET_WM_HANDLED_ICONS,
        _NET_WM_ICON,
        _NET_WM_ICON_GEOMETRY,
        _NET_WM_ICON_NAME,
        _NET_WM_MOVERESIZE,
        _NET_WM_NAME,
        _NET_WM_OPAQUE_REGION,
        _NET_WM_PID,
        _NET_WM_PING,
        _NET_WM_WINDOW_OPACITY,
        _NET_WM_WINDOW_OPACITY_LOCKED,
        _NET_WM_STATE,
        _NET_WM_STATE_ABOVE,
        _NET_WM_STATE_BELOW,
        _NET_WM_STATE_DEMANDS_ATTENTION,
        _NET_WM_STATE_FOCUSED,
        _NET_WM_STATE_FULLSCREEN,
        _NET_WM_STATE_HIDDEN,
        _NET_WM_STATE_MAXIMIZED_VERT,
        _NET_WM_STATE_MAXIMIZED_HORZ,
        _NET_WM_STATE_MODAL,
        _NET_WM_STATE_SHADED,
        _NET_WM_STATE_SKIP_PAGER,
        _NET_WM_STATE_SKIP_TASKBAR,
        _NET_WM_STATE_STICKY,
        _NET_WM_STRUT,
        _NET_WM_STRUT_PARTIAL,
        _NET_WM_SYNC_REQUEST,
        _NET_WM_SYNC_REQUEST_COUNTER,
        _NET_WM_USER_TIME,
        _NET_WM_USER_TIME_WINDOW,
        _NET_WM_VISIBLE_NAME,
        _NET_WM_VISIBLE_ICON_NAME,
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
        _NET_WORKAREA,
        UTF8_STRING,
        WM_CLASS,
    }
}

// Window Manager control provides a simplified access layer to the EWMH functions exposed
// through the x11 libraries.
pub(crate) struct WmCtl
{
    conn: RustConnection,               // x11 connection
    atoms: AtomCollection,              // atom cache
    supported: HashMap<u32, String>,    // cache for supported functions
    pub(crate) screen: usize,           // screen number
    pub(crate) root: u32,               // root window id
    pub(crate) width: u16,              // screen width
    pub(crate) height: u16,             // screen height
    pub(crate) work_width: u16,         // screen height
    pub(crate) work_height: u16,        // screen height
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

        // Get the screen size
        let (width, height, root) = {
            let screen = &conn.setup().roots[screen];
            (screen.width_in_pixels, screen.height_in_pixels, screen.root)
        };

        // Populate the supported functions cache
        let (atoms, supported) = WmCtl::init_caching(&conn, root)?;

        // Create the window manager object
        let mut wmctl = WmCtl{
            conn, atoms, supported,
            screen, root, width, height,
            work_width: Default::default(),
            work_height: Default::default(),
        };

        // Get the work area
        let (width, height) = wmctl.workarea()?;
        wmctl.work_width = width;
        wmctl.work_height = height;

        debug!("connect: screen: {}, root: {}, w: {}, h: {}", screen, root, width, height);
        Ok(wmctl)
    }

    fn init_caching(conn: &RustConnection, root: u32) -> WmCtlResult<(AtomCollection, HashMap<u32, String>)> {
        debug!("initializing caching...");

        // Cache atoms
        let atoms = AtomCollection::new(conn)?.reply()?;

        // Cache supported functions
        let mut supported = HashMap::<u32, String>::new();
        let reply = conn.get_property(false, root, atoms._NET_SUPPORTED, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
        for atom in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_SUPPORTED".to_owned()))? {
            if let Ok(name) = atom_to_string(&atoms, atom) {
                trace!("supported: {}", atom_to_string(&atoms, atom)?);
                supported.insert(atom, name);
            }
        }
        debug!("caching initialized");
        Ok((atoms, supported))
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

    // Determine if the given function is supported by the window manager
    // Defined as: _NET_SUPPOTED, ATOM[]/32
    #[allow(dead_code)]
    pub(crate) fn supported(&self, atom: u32) -> bool {
        self.supported.get(&atom).is_some()
    }

    // Get windows optionally all
    pub(crate) fn windows(&self, all: bool) -> WmCtlResult<Vec<u32>> {
        let mut windows = vec![];
        if all {
            // All windows in the X11 system
            let tree = self.query_tree(self.root)?.reply()?;
            for win in tree.children {
                windows.push(win);
            }
        } else {
            // Window manager client windows which is a subset of all windows that have been
            // reparented i.e. new ids and don't map to the same ids as their all windows selves.
            let reply = self.get_property(false, self.root, self.atoms._NET_CLIENT_LIST, AtomEnum::WINDOW, 0, u32::MAX)?.reply()?;
            for win in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_CLIENT_LIST".to_owned()))? {
                windows.push(win)
            }
        }
        Ok(windows)
    }

    // Get window manager's window id and name
    pub(crate) fn winmgr(&self) -> WmCtlResult<(u32, String)> {
        let reply = self.get_property(false, self.root, self.atoms._NET_SUPPORTING_WM_CHECK, AtomEnum::WINDOW, 0, u32::MAX)?.reply()?;
        let win = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_SUPPORTING_WM_CHECK".to_owned()))?;
        let name = self.win_name(win)?;
        debug!("winmgr: id: {}, name: {}", win, name);
        Ok((win, name))
    }

    // Get desktop work area
    // Defined as: _NET_WORKAREA, x, y, width, height CARDINAL[][4]/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WORKAREA`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be 4 for each desktop as defined (x, y, width, height).
    pub(crate) fn workarea(&self) -> WmCtlResult<(u16, u16)> {
        let reply = self.get_property(false, self.root, self.atoms._NET_WORKAREA, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let mut values = reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA".to_owned()))?;
        let x = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA x".to_owned()))?;
        let y = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA y".to_owned()))?;
        let w = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA width".to_owned()))?;
        let h = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_WORKAREA height".to_owned()))?;
        debug!("work_area: x: {}, y: {}, w: {}, h: {}", x, y, w, h);

        // x and y are always zero so dropping them
        Ok((w as u16, h as u16))
    }

    // Get window attribrtes
    pub(crate) fn win_attributes(&self, win: xproto::Window) -> WmCtlResult<(WinClass, WinMap)> {
        let attr = self.get_window_attributes(win)?.reply()?;
        debug!("win_attributes: id: {}, class: {:?}, state: {:?}", win, attr.class, attr.map_state);
        Ok((WinClass::from(attr.class.into())?, WinMap::from(attr.map_state.into())?))
    }

    // Get window class which ends up being the applications name
    pub(crate) fn win_class(&self, win: xproto::Window) -> WmCtlResult<String> {
        let reply = self.get_property(false, win, self.atoms.WM_CLASS, AtomEnum::STRING, 0, u32::MAX)?.reply()?;

        // Skip the first null terminated string
        let iter = reply.value.into_iter().skip_while(|x| *x != 0).skip(1);

        // Extract the second null terminated string
        let class = str::from_utf8(&iter.take_while(|x| *x != 0).collect::<Vec<_>>())?.to_owned();
        debug!("win_class: id: {}, class: {}", win, class);
        Ok(class)
    }

    // Get window desktop
    // Defined as: _NET_WM_DESKTOP desktop, CARDINAL/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_DESKTOP`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be a single value.
    pub(crate) fn win_desktop(&self, win: xproto::Window) -> WmCtlResult<i32> {
        let reply = self.get_property(false, win, self.atoms._NET_WM_DESKTOP, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let desktop = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_WM_DESKTOP".to_owned()))?;
        debug!("win_desktop: id: {}, desktop: {}", win, desktop);
        Ok(desktop as i32)
    }

    // Get window frame border values added by the window manager
    // Defined as: _NET_FRAME_EXTENTS, left, right, top, bottom, CARDINAL[4]/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_FRAME_EXTENTS`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be...
    pub(crate) fn win_borders(&self, win: xproto::Window) -> WmCtlResult<(i32, i32, i32, i32)> {
        let reply = self.get_property(false, win, self.atoms._NET_FRAME_EXTENTS, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let mut values = reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS".to_owned()))?;
        let l = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS left".to_owned()))?;
        let r = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS right".to_owned()))?;
        let t = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS top".to_owned()))?;
        let b = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS bottom".to_owned()))?;
        debug!("win_extents: id: {}, l: {}, r: {}, t: {}, b: {}", win, l, r, t, b);
        Ok((l as i32, r as i32, t as i32, b as i32))
    }

    // Get window geometry
    pub(crate) fn win_geometry(&self, win: xproto::Window) -> WmCtlResult<(i32, i32, u32, u32)> {

        // The returned x, y location is relative to its parent window making the values completely
        // useless. However using `translate_coordinates` we can have the window manager map those
        // useless values into real world cordinates by passing it the root as the relative window.

        // Get width and heith and useless relative location values
        let g = self.get_geometry(win)?.reply()?;

        // Translate the useless retative location values to to real world values
        let t = self.translate_coordinates(win, self.root, g.x, g.y)?.reply()?;

        let (x, y, w, h) = (t.dst_x, t.dst_y, g.width, g.height);
        debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", win, x, y, w, h);
        Ok((x as i32, y as i32, w as u32, h as u32))
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

    // Get window parent
    #[allow(dead_code)]
    pub(crate) fn win_parent(&self, win: xproto::Window) -> WmCtlResult<u32> {
        let tree = self.query_tree(win)?.reply()?;
        let id = tree.parent;
        debug!("win_parent: id: {}, parent: {:?}", win, id);
        Ok(id)
    }

    // Get window pid
    // Defined as: _NET_WM_PID, CARDINAL/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_PID`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be a single value.
    pub(crate) fn win_pid(&self, win: xproto::Window) -> WmCtlResult<i32> {
        let reply = self.get_property(false, win, self.atoms._NET_WM_PID, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let pid = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_WM_PID".to_owned()))?;
        debug!("win_pid: id: {}, pid: {:?}", win, pid);
        Ok(pid as i32)
    }

    // Get window state
    // Defined as: _NET_WM_STATE, ATOM[]
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_STATE`
    // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be a single value.
    pub(crate) fn win_state(&self, win: xproto::Window) -> WmCtlResult<Vec<WinState>> {
        let mut states = vec![];
        let reply = self.get_property(false, win, self.atoms._NET_WM_STATE, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
        for state in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_WM_STATE".to_owned()))? {
            let state = WinState::from(&self.atoms, state)?;
            debug!("win_state: id: {}, state: {}", win, state);
            states.push(state);
        }
        Ok(states)
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

    // Helper method to print out the data type
    // println!("DataType NET: {:?}", AtomEnum::from(reply.type_ as u8));
    #[allow(dead_code)]
    pub(crate) fn print_data_type(reply: &GetPropertyReply) {
        println!("DataType: {:?}", AtomEnum::from(reply.type_ as u8));
    }
}
