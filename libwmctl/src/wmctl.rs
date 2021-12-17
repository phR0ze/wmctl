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
use crate::{WmCtlResult, WmCtlError, model::*};
use std::{str, sync::Arc, collections::HashMap};
use tracing::{trace, debug};

use x11rb::{
    atom_manager,
    connection::{Connection},
    protocol::{xproto::{ConnectionExt as _, self, *}},
    rust_connection::RustConnection,
};

// A collection of the atoms we will need.
atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
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
    }
}

// Define the second byte of the move resize flags 32bit value
// Used to indicate that the associated value has been changed and needs to be acted upon
type MoveResizeWindowFlags = u32;
const MOVE_RESIZE_WINDOW_X:         MoveResizeWindowFlags = 1 << 8;
const MOVE_RESIZE_WINDOW_Y:         MoveResizeWindowFlags = 1 << 9;
const MOVE_RESIZE_WINDOW_WIDTH:     MoveResizeWindowFlags = 1 << 10;
const MOVE_RESIZE_WINDOW_HEIGHT:    MoveResizeWindowFlags = 1 << 11;

type WindowStateAction = u32;
const WINDOW_STATE_ACTION_REMOVE:   WindowStateAction = 0;
const WINDOW_STATE_ACTION_ADD:      WindowStateAction = 1;

/// Window Manager control implements the EWMH protocol using x11rb to provide a simplified access
/// layer to EWHM compatible window managers.
pub struct WmCtl
{
    conn: Arc<RustConnection>,          // x11 connection
    atoms: AtomCollection,              // atom cache
    supported: HashMap<u32, bool>,      // cache for supported functions
    pub(crate) screen: usize,           // screen number
    pub(crate) root: u32,               // root window id
    pub(crate) width: u32,              // screen width
    pub(crate) height: u32,             // screen height
    pub(crate) work_width: u32,         // screen height
    pub(crate) work_height: u32,        // screen height
}

impl WmCtl
{
    /// Create the window manager control instance and connect to the X11 server
    pub fn connect() -> WmCtlResult<Self>
    {
        let (conn, screen) = x11rb::connect(None)?;

        // Get the screen size
        let (width, height, root) = {
            let screen = &conn.setup().roots[screen];
            (screen.width_in_pixels as u32, screen.height_in_pixels as u32, screen.root)
        };

        // Populate the supported functions cache
        let (atoms, supported) = WmCtl::init_caching(&conn, root)?;

        // Create the window manager object
        let mut wmctl = WmCtl{
            conn: Arc::new(conn),
            atoms, supported,
            screen, root, width, height,
            work_width: Default::default(),
            work_height: Default::default(),
        };

        // Get the work area
        let (width, height) = wmctl.workarea()?;
        wmctl.work_width = width as u32;
        wmctl.work_height = height as u32;

        debug!("connect: screen: {}, root: {}, w: {}, h: {}", screen, root, width, height);
        Ok(wmctl)
    }

    fn init_caching(conn: &RustConnection, root: u32) -> WmCtlResult<(AtomCollection, HashMap<u32, bool>)>
    {
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

    /// Get the active window id
    pub fn active_win(&self) -> WmCtlResult<u32>
    {
        // Defined as: _NET_ACTIVE_WINDOW, WINDOW/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_ACTIVE_WINDOW`
        // request message with a `AtomEnum::WINDOW` type response and we can use the `reply.value32()` accessor to
        // retrieve the value.
        let reply = self.conn.get_property(false, self.root, self.atoms._NET_ACTIVE_WINDOW, AtomEnum::WINDOW, 0, u32::MAX)?.reply()?;
        let win = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_ACTIVE_WINDOW".to_owned()))?;
        debug!("active_win: {}", win);
        Ok(win)
    }

    /// Check if a composit manager is running
    pub fn composite_manager(&self) -> WmCtlResult<bool>
    {
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

    // Get number of desktops
    pub fn desktops(&self) -> WmCtlResult<u32>
    {
        // Defined as: _NET_NUMBER_OF_DESKTOPS, CARDINAL/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_NUMBER_OF_DESKTOPS`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the value.
        let reply = self.conn.get_property(false, self.root, self.atoms._NET_NUMBER_OF_DESKTOPS, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let num = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_NUMBER_OF_DESKTOPS".to_owned()))?;
        debug!("desktops: {}", num);
        Ok(num)
    }

    /// Add the MaxVert and MaxHorz states
    pub fn maximize_win(&self, win: xproto::Window) -> WmCtlResult<()>
    {  
        self.send_event(ClientMessageEvent::new(32, win, self.atoms._NET_WM_STATE, [
            WINDOW_STATE_ACTION_ADD,
            self.atoms._NET_WM_STATE_MAXIMIZED_HORZ,
            self.atoms._NET_WM_STATE_MAXIMIZED_VERT,
            0, 0]))?;
        debug!("maximize: id: {}", win);
        Ok(())
    }

    /// Move and resize the given window
    pub fn move_resize_win(&self, win: xproto::Window, gravity: Option<u32>,
        x: Option<u32>, y: Option<u32>, w: Option<u32>, h: Option<u32>) -> WmCtlResult<()>
    {
        // Construct the move resize message 
        //
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

        self.send_event(ClientMessageEvent::new(32, win, self.atoms._NET_MOVERESIZE_WINDOW,
            [flags, x.unwrap_or(0), y.unwrap_or(0), w.unwrap_or(0), h.unwrap_or(0)]))?;

        debug!("move_resize_win: id: {}, g: {:?}, x: {:?}, y: {:?}, w: {:?}, h: {:?}", win, gravity, x, y, w, h);
        Ok(())
    }

    // Send the event ensuring that a flush is called and that the message was precisely
    // executed in the case of a resize/move.
    fn send_event(&self, msg: ClientMessageEvent) -> WmCtlResult<()>
    {
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

    // Determine if the given function is supported by the window manager
    #[allow(dead_code)]
    pub fn supported(&self, atom: u32) -> bool
    {
        self.supported.get(&atom).is_some()
    }

    /// Remove the MaxVert and MaxHorz states
    pub fn unmaximize_win(&self, win: xproto::Window) -> WmCtlResult<()>
    {  
        self.send_event(ClientMessageEvent::new(32, win, self.atoms._NET_WM_STATE, [
            WINDOW_STATE_ACTION_REMOVE,
            self.atoms._NET_WM_STATE_MAXIMIZED_HORZ,
            self.atoms._NET_WM_STATE_MAXIMIZED_VERT,
            0, 0]))?;
        debug!("unmaximize: id: {}", win);
        Ok(())
    }

    /// Get windows optionally all
    pub fn windows(&self, all: bool) -> WmCtlResult<Vec<u32>>
    {
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
            let reply = self.conn.get_property(false, self.root, self.atoms._NET_CLIENT_LIST, AtomEnum::WINDOW, 0, u32::MAX)?.reply()?;
            for win in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_CLIENT_LIST".to_owned()))? {
                windows.push(win)
            }
        }
        Ok(windows)
    }

    /// Get window manager's window id and name
    pub fn winmgr(&self) -> WmCtlResult<(u32, String)>
    {
        let reply = self.conn.get_property(false, self.root, self.atoms._NET_SUPPORTING_WM_CHECK, AtomEnum::WINDOW, 0, u32::MAX)?.reply()?;
        let win = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_SUPPORTING_WM_CHECK".to_owned()))?;
        let name = self.win_name(win)?;
        debug!("winmgr: id: {}, name: {}", win, name);
        Ok((win, name))
    }

    /// Get desktop work area
    pub fn workarea(&self) -> WmCtlResult<(u16, u16)>
    {
        // Defined as: _NET_WORKAREA, x, y, width, height CARDINAL[][4]/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WORKAREA`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be 4 for each desktop as defined (x, y, width, height).
        let reply = self.conn.get_property(false, self.root, self.atoms._NET_WORKAREA, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
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
    #[allow(dead_code)]
    pub fn win_attributes(&self, win: xproto::Window) -> WmCtlResult<(WinClass, WinMap)>
    {
        let attr = self.conn.get_window_attributes(win)?.reply()?;
        debug!("win_attributes: id: {}, win_gravity: {:?}, bit_gravity: {:?}", win, attr.win_gravity, attr.bit_gravity);
        Ok((WinClass::from(attr.class.into())?, WinMap::from(attr.map_state.into())?))
    }

    // Get window class which ends up being the applications name
    pub(crate) fn win_class(&self, win: xproto::Window) -> WmCtlResult<String>
    {
        let reply = self.conn.get_property(false, win, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, u32::MAX)?.reply()?;

        // Skip the first null terminated string and extract the second
        let iter = reply.value.into_iter().skip_while(|x| *x != 0).skip(1).take_while(|x| *x != 0);

        // Extract the second null terminated string
        let class = str::from_utf8(&iter.collect::<Vec<_>>())?.to_owned();
        debug!("win_class: id: {}, class: {}", win, class);
        Ok(class)
    }

    // Get window desktop
    // Defined as: _NET_WM_DESKTOP desktop, CARDINAL/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_DESKTOP`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be a single value.
    pub(crate) fn win_desktop(&self, win: xproto::Window) -> WmCtlResult<i32>
    {
        let reply = self.conn.get_property(false, win, self.atoms._NET_WM_DESKTOP, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let desktop = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_WM_DESKTOP".to_owned()))?;
        debug!("win_desktop: id: {}, desktop: {}", win, desktop);
        Ok(desktop as i32)
    }

    // Get window frame border values added by the window manager
    // Defined as: _NET_FRAME_EXTENTS, left, right, top, bottom, CARDINAL[4]/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_FRAME_EXTENTS`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be...
    pub(crate) fn win_borders(&self, win: xproto::Window) -> WmCtlResult<(u32, u32, u32, u32)>
    {
        let reply = self.conn.get_property(false, win, self.atoms._NET_FRAME_EXTENTS, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let mut values = reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS".to_owned()))?;
        let l = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS left".to_owned()))?;
        let r = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS right".to_owned()))?;
        let t = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS top".to_owned()))?;
        let b = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS bottom".to_owned()))?;
        debug!("win_borders: id: {}, l: {}, r: {}, t: {}, b: {}", win, l, r, t, b);
        Ok((l, r, t, b))
    }

    // Get window geometry
    pub(crate) fn win_geometry(&self, win: xproto::Window) -> WmCtlResult<(i32, i32, u32, u32)>
    {
        // The returned x, y location is relative to its parent window making the values completely
        // useless. However using `translate_coordinates` we can have the window manager map those
        // useless values into real world cordinates by passing it the root as the relative window.

        // Get width and heith and useless relative location values
        let g = self.conn.get_geometry(win)?.reply()?;

        // Translate the useless retative location values to to real world values
        let t = self.conn.translate_coordinates(win, self.root, g.x, g.y)?.reply()?;

        let (x, y, w, h) = (t.dst_x, t.dst_y, g.width, g.height);
        debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", win, x, y, w, h);
        Ok((x as i32, y as i32, w as u32, h as u32))
    }

    // Get window name
    // Defined as: _NET_WM_NAME, UTF8_STRING
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_NAME`
    // request message with a `AtomEnum::UTF8_STRING` type response and we can use the `reply.value` accessor to
    // retrieve the value.
    pub(crate) fn win_name(&self, win: xproto::Window) -> WmCtlResult<String>
    {
        // First try the _NET_WM_VISIBLE_NAME
        let reply = self.conn.get_property(false, win, self.atoms._NET_WM_VISIBLE_NAME, self.atoms.UTF8_STRING, 0, u32::MAX)?.reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using _NET_WM_VISIBLE_NAME for: {}", value);
                    return Ok(value.to_owned())
                }
            }
        }

        // Next try the _NET_WM_NAME
        let reply = self.conn.get_property(false, win, self.atoms._NET_WM_NAME, self.atoms.UTF8_STRING, 0, u32::MAX)?.reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using _NET_WM_NAME for: {}", value);
                    return Ok(value.to_owned())
                }
            }
        }

        // Fall back on the WM_NAME
        let reply = self.conn.get_property(false, win, AtomEnum::WM_NAME, AtomEnum::STRING, 0, u32::MAX)?.reply()?;
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
    pub(crate) fn win_parent(&self, win: xproto::Window) -> WmCtlResult<u32>
    {
        let tree = self.conn.query_tree(win)?.reply()?;
        let id = tree.parent;
        debug!("win_parent: id: {}, parent: {:?}", win, id);
        Ok(id)
    }

    // Get window pid
    // Defined as: _NET_WM_PID, CARDINAL/32
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_PID`
    // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be a single value.
    pub(crate) fn win_pid(&self, win: xproto::Window) -> WmCtlResult<i32>
    {
        let reply = self.conn.get_property(false, win, self.atoms._NET_WM_PID, AtomEnum::CARDINAL, 0, u32::MAX)?.reply()?;
        let pid = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_WM_PID".to_owned()))?;
        debug!("win_pid: id: {}, pid: {:?}", win, pid);
        Ok(pid as i32)
    }

    // Get window state
    // Defined as: _NET_WM_STATE, ATOM[]
    // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_STATE`
    // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
    // retrieve the values of which there will be a single value.
    pub(crate) fn win_state(&self, win: xproto::Window) -> WmCtlResult<Vec<WinState>>
    {
        let mut states = vec![];
        let reply = self.conn.get_property(false, win, self.atoms._NET_WM_STATE, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
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
    pub(crate) fn win_type(&self, win: xproto::Window) -> WmCtlResult<WinType>
    {
        let reply = self.conn.get_property(false, win, self.atoms._NET_WM_WINDOW_TYPE, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
        let typ = reply.value32().and_then(|mut x| x.next()).ok_or(WmCtlError::PropertyNotFound("_NET_WM_WINDOW_TYPE".to_owned()))?;
        let typ = WinType::from(&self.atoms, typ)?;
        debug!("win_type: id: {}, type: {:?}", win, typ);
        Ok(typ)
    }

    // Helper method to print out the data type
    // println!("DataType NET: {:?}", AtomEnum::from(reply.type_ as u8));
    #[allow(dead_code)]
    pub(crate) fn print_data_type(reply: &GetPropertyReply)
    {
        println!("DataType: {:?}", AtomEnum::from(reply.type_ as u8));
    }
}
