use crate::{model::*, WmCtlError, WmCtlResult, WMCTL};
use std::str;
use tracing::debug;
use x11rb::protocol::xproto::{ConnectionExt as _, *};

/// Window provides information about a specific window.
/// - geometry: (x, y, width, height)
/// - borders: (left, right, top, bottom)
pub struct Window {
    pub id: u32,

    // Directives
    shape: Option<Shape>,
    pos: Option<Position>,
}

impl Window {
    pub(crate) fn new(id: u32) -> Self {
        Self {
            id,
            shape: None,
            pos: None,
        }
    }

    /// Use the given window id or the active window id if none is provided
    pub(crate) fn from(id: Option<u32>) -> Self {
        Self {
            id: id.unwrap_or(WMCTL().read().unwrap().active_window().unwrap()),
            shape: None,
            pos: None,
        }
    }

    /// Get window pid
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let pid = win.pid().unwrap();
    /// ```
    pub fn pid(&self) -> WmCtlResult<i32> {
        let wm = WMCTL().read().unwrap();

        // Defined as: _NET_WM_PID, CARDINAL/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_PID`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be a single value.
        let reply = wm
            .conn
            .get_property(false, self.id, wm.atoms._NET_WM_PID, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;
        let pid = reply
            .value32()
            .and_then(|mut x| x.next())
            .ok_or(WmCtlError::PropertyNotFound("_NET_WM_PID".to_owned()))?;
        debug!("win_pid: id: {}, pid: {:?}", self.id, pid);
        Ok(pid as i32)
    }

    /// Get window name
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let name = win.name().unwrap();
    /// ```
    pub fn name(&self) -> WmCtlResult<String> {
        let wm = WMCTL().read().unwrap();

        // Defined as: _NET_WM_NAME, UTF8_STRING
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_NAME`
        // request message with a `AtomEnum::UTF8_STRING` type response and we can use the `reply.value` accessor to
        // retrieve the value.

        // First try the _NET_WM_VISIBLE_NAME
        let reply = wm
            .conn
            .get_property(false, self.id, wm.atoms._NET_WM_VISIBLE_NAME, wm.atoms.UTF8_STRING, 0, u32::MAX)?
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
        let reply = wm
            .conn
            .get_property(false, self.id, wm.atoms._NET_WM_NAME, wm.atoms.UTF8_STRING, 0, u32::MAX)?
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
            wm.conn.get_property(false, self.id, AtomEnum::WM_NAME, AtomEnum::STRING, 0, u32::MAX)?.reply()?;
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
    /// let win = window(12345);
    /// let class = win.class().unwrap();
    /// ```
    pub fn class(&self) -> WmCtlResult<String> {
        let wm = WMCTL().read().unwrap();

        let reply =
            wm.conn.get_property(false, self.id, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, u32::MAX)?.reply()?;

        // Skip the first null terminated string and extract the second
        let iter = reply.value.into_iter().skip_while(|x| *x != 0).skip(1).take_while(|x| *x != 0);

        // Extract the second null terminated string
        let class = str::from_utf8(&iter.collect::<Vec<_>>())?.to_owned();
        debug!("win_class: id: {}, class: {}", self.id, class);
        Ok(class)
    }

    /// Get window kind
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let kind = win.kind().unwrap();
    /// ```
    pub fn kind(&self) -> WmCtlResult<WinKind> {
        let wm = WMCTL().read().unwrap();

        // Defined as: _NET_WM_WINDOW_TYPE, ATOM[]/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_WINDOW_TYPE`
        // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
        // retrieve the value.
        let reply = wm
            .conn
            .get_property(false, self.id, wm.atoms._NET_WM_WINDOW_TYPE, AtomEnum::ATOM, 0, u32::MAX)?
            .reply()?;
        let typ = reply
            .value32()
            .and_then(|mut x| x.next())
            .ok_or(WmCtlError::PropertyNotFound("_NET_WM_WINDOW_TYPE".to_owned()))?;
        let _kind = WinKind::from(&wm.atoms, typ)?;
        debug!("win_kind: id: {}, kind: {:?}", self.id, _kind);
        Ok(_kind)
    }

    /// Get window state
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let state = win.state().unwrap();
    /// ```
    pub fn state(&self) -> WmCtlResult<Vec<WinState>> {
        let wm = WMCTL().read().unwrap();

        // Defined as: _NET_WM_STATE, ATOM[]
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_STATE`
        // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be a single value.
        let mut states = vec![];
        let reply =
            wm.conn.get_property(false, self.id, wm.atoms._NET_WM_STATE, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
        for state in reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_WM_STATE".to_owned()))? {
            let state = WinState::from(&wm.atoms, state)?;
            debug!("win_state: id: {}, state: {}", self.id, state);
            states.push(state);
        }
        Ok(states)
    }

    /// Get window parent
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let parent = win.parent().unwrap();
    /// ```
    #[allow(dead_code)]
    pub fn parent(&self) -> WmCtlResult<Window> {
        let wm = WMCTL().read().unwrap();

        let tree = wm.conn.query_tree(self.id)?.reply()?;
        let parent_id = tree.parent;
        debug!("win_parent: id: {}, parent: {:?}", self.id, parent_id);
        Ok(Window::new(parent_id))
    }

    /// Get window desktop
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let desktop = win.desktop().unwrap();
    /// ```
    pub fn desktop(&self) -> WmCtlResult<i32> {
        let wm = WMCTL().read().unwrap();

        // Defined as: _NET_WM_DESKTOP desktop, CARDINAL/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_DESKTOP`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be a single value.
        let reply = wm
            .conn
            .get_property(false, self.id, wm.atoms._NET_WM_DESKTOP, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;
        let desktop = reply
            .value32()
            .and_then(|mut x| x.next())
            .ok_or(WmCtlError::PropertyNotFound("_NET_WM_DESKTOP".to_owned()))?;
        debug!("win_desktop: id: {}, desktop: {}", self.id, desktop);
        Ok(desktop as i32)
    }

    /// Get window geometry
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let (x, y, w, h) = win.geometry().unwrap();
    /// ```
    pub fn geometry(&self) -> WmCtlResult<(i32, i32, u32, u32)> {
        let wm = WMCTL().read().unwrap();

        // The returned x, y location is relative to its parent window making the values completely
        // useless. However using `translate_coordinates` we can have the window manager map those
        // useless values into real world cordinates by passing it the root as the relative window.

        // Get width and heith and useless relative location values
        let g = wm.conn.get_geometry(self.id)?.reply()?;

        // Translate the useless retative location values to to real world values
        let t = wm.conn.translate_coordinates(self.id, wm.root, g.x, g.y)?.reply()?;

        let (x, y, w, h) = (t.dst_x, t.dst_y, g.width, g.height);
        debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", self.id, x, y, w, h);
        Ok((x as i32, y as i32, w as u32, h as u32))
    }

    /// Get window frame border values added by the window manager
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let (l, r, t, b) = win.borders().unwrap();
    /// ```
    pub fn borders(&self) -> WmCtlResult<(u32, u32, u32, u32)> {
        let wm = WMCTL().read().unwrap();

        // Defined as: _NET_FRAME_EXTENTS, left, right, top, bottom, CARDINAL[4]/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_FRAME_EXTENTS`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be...
        let reply = wm
            .conn
            .get_property(false, self.id, wm.atoms._NET_FRAME_EXTENTS, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;
        let mut values = reply.value32().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS".to_owned()))?;
        let l = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS left".to_owned()))?;
        let r = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS right".to_owned()))?;
        let t = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS top".to_owned()))?;
        let b = values.next().ok_or(WmCtlError::PropertyNotFound("_NET_FRAME_EXTENTS bottom".to_owned()))?;
        debug!("win_borders: id: {}, l: {}, r: {}, t: {}, b: {}", self.id, l, r, t, b);
        Ok((l, r, t, b))
    }

    /// Maximize the window both horizontally and vertically
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// win.maximize().unwrap();
    /// ```
    pub fn maximize(&self) -> WmCtlResult<()> {
        let wm = WMCTL().read().unwrap();

        wm.send_event(ClientMessageEvent::new(
            32,
            self.id,
            wm.atoms._NET_WM_STATE,
            [
                WINDOW_STATE_ACTION_ADD,
                wm.atoms._NET_WM_STATE_MAXIMIZED_HORZ,
                wm.atoms._NET_WM_STATE_MAXIMIZED_VERT,
                0,
                0,
            ],
        ))?;
        debug!("maximize: id: {}", self.id);
        Ok(())
    }

    /// Check if the window has a horizontally or vertically maximized
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// win.maximized()
    /// ```
    pub fn maximized(&self) -> bool {
        self.state().is_ok_and(|states| states.contains(&WinState::MaxVert) || states.contains(&WinState::MaxHorz))
    }

    /// Remove the MaxVert and MaxHorz states
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// win.unmaximize().unwrap();
    /// ```
    pub fn unmaximize(&self) -> WmCtlResult<()> {
        let wm = WMCTL().read().unwrap();

        wm.send_event(ClientMessageEvent::new(
            32,
            self.id,
            wm.atoms._NET_WM_STATE,
            [
                WINDOW_STATE_ACTION_REMOVE,
                wm.atoms._NET_WM_STATE_MAXIMIZED_HORZ,
                wm.atoms._NET_WM_STATE_MAXIMIZED_VERT,
                0,
                0,
            ],
        ))?;
        debug!("unmaximize: id: {}", self.id);
        Ok(())
    }

    /// Queue the shape the window should be. This will not take effect until the place() method is called.
    ///
    /// ### Arguments
    /// * `shape` - pre-defined shape to manipulate the window into
    ///
    /// ### Examples
    /// ```
    /// use libwmctl::prelude::*;
    /// window(12345).shape(WinShape::Large).place().unwrap();
    /// ```
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Queue the position the window should be. This will not take effect until the place() method is called.
    ///
    /// ### Arguments
    /// * `pos` - pre-defined position to move the window to
    ///
    /// ### Examples
    /// ```
    /// use libwmctl::prelude::*;
    /// window(12345).pos(WinPosition::Right).place().unwrap();
    /// ```
    pub fn pos(mut self, pos: Position) -> Self {
        self.pos = Some(pos);
        self
    }

    /// Move and resize the window according to the queued directives configured with the shape()
    /// and pos() methods.
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// win.shape(Shape::Large).pos(Position::Right).place();
    /// ```
    pub fn place(&self) -> WmCtlResult<()> {
        if self.shape.is_none() && self.pos.is_none() {
            return Ok(());
        }
        let wm = WMCTL().read().unwrap();

        // Unmaximize to shape and position the window correctly
        //self.unmaximize()?;

        // Get window properties
        let (bl, br, bt, bb) = self.borders()?;
        let (_, _, w, h) = self.geometry()?;

        // Shape the window as directed
        let (gravity, sw, sh) = if let Some(_shape) = self.shape {
            let (gravity, sw, sh) = translate_shape(w, h, bl + br, bt + bb, _shape)?;

            // Don't use gravity if positioning is required
            if self.pos.is_some() || self.x.is_some() || self.y.is_some() {
                (None, sw, sh)
            } else {
                (gravity, sw, sh)
            }
        } else if self.w.is_some() && self.h.is_some() {
            (None, Some(self.w.unwrap()), Some(self.h.unwrap()))
        } else {
            (None, None, None)
        };

        // Position the window if directed
        let (x, y) = if let Some(pos) = &self.pos {
            translate_pos(w, h, bl + br, bt + bb, wm.work_width, wm.work_height, pos)?
        } else {
            (None, None)
        };

        // Execute if reason to
        self.move_resize(gravity, x, y, sw, sh)
    }

    /// Move and resize
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
    /// let win = window(12345);
    /// win.move_resize_win(None, Some(0), Some(0), Some(500), Some(500)).unwrap();
    /// ```
    fn move_resize(
        &self, gravity: Option<u32>, x: Option<u32>, y: Option<u32>, w: Option<u32>, h: Option<u32>,
    ) -> WmCtlResult<()> {
        let wm = WMCTL().read().unwrap();

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

        wm.send_event(ClientMessageEvent::new(
            32,
            self.id,
            wm.atoms._NET_MOVERESIZE_WINDOW,
            [flags, x.unwrap_or(0), y.unwrap_or(0), w.unwrap_or(0), h.unwrap_or(0)],
        ))?;

        debug!("move_resize: id: {}, g: {:?}, x: {:?}, y: {:?}, w: {:?}, h: {:?}", self.id, gravity, x, y, w, h);
        Ok(())
    }
}

/// Translate position enum values into (x, y) cordinates but takes no direct action on the window.
/// Window should already be unmaximized before calling this function.
///
/// ### Arguments
/// * `w` - Window's current width
/// * `h` - Window's current height
/// * `bw` - Window's border width
/// * `bh` - Window's border height
/// * `aw` - Window manager's work area width
/// * `ah` - Window manager's work area height
/// * `pos` - Position to translate
///
/// ### Returns
/// * `(x, y)` cordinates or (None, None) for no change
fn translate_pos(
    w: u32, h: u32, bw: u32, bh: u32, aw: u32, ah: u32, pos: &Position,
) -> WmCtlResult<(Option<u32>, Option<u32>)> {
    // Pre-calculating some commonly used values for the translation

    // x center coordinate for left of window such that the window will appear horizontally centered
    //
    // * if half the window+border is more than half the work area then it will be off the screen
    //   so use 0 instead so that the window is flush with the edge and still usable.
    // * else calculate half the work area minus half the window+border to get the x coordinate
    let cx = if (w + bw) / 2 >= aw / 2 { 0 } else { (aw as f32 / 2.0 - (w + bw) as f32 / 2.0) as u32 };

    // y center coordinate for top of window such that the window will appear vertically centered
    //
    // * if half the window+border is more than half the work area then it will be off the screen
    //   so use 0 instead so that the window is flush with the edge and still usable.
    // * else calculate half the work area minus half the window+border to get the y coordinate
    let cy = if (h + bh) / 2 >= ah / 2 { 0 } else { (ah as f32 / 2.0 - (h + bh) as f32 / 2.0) as u32 };

    // x left coordinate for the window such that the window will appear all the way to the right
    //
    // * if the window+border is more than the work area then it will be off the screen
    //   so use 0 instead so that the window is flush with the edge and still usable.
    // * else calculate the window+border minus the work area to get the x coordinate
    let lx = if w + bw >= aw { 0 } else { aw - w - bw };

    // y top coordinate for the window such that the window will appear all the way to the top
    //
    // * if the window+border is more than the work area then it will be off the screen
    //   so use 0 instead so that the window is flush with the edge and still usable.
    // * else calculate the window+border minus the work area to get the y coordinate
    let ty = if h + bh >= ah { 0 } else { ah - h - bh };

    Ok(match pos {
        Position::Center => (Some(cx), Some(cy)),
        Position::Left => (Some(0), None),
        Position::Right => (Some(lx), None),
        Position::Top => (None, Some(0)),
        Position::Bottom => (None, Some(ty)),
        Position::TopLeft => (Some(0), Some(0)),
        Position::TopRight => (Some(lx), Some(0)),
        Position::BottomLeft => (Some(0), Some(ty)),
        Position::BottomRight => (Some(lx), Some(ty)),
        Position::LeftCenter => (Some(0), Some(cy)),
        Position::RightCenter => (Some(lx), Some(cy)),
        Position::TopCenter => (Some(cx), Some(0)),
        Position::BottomCenter => (Some(cx), Some(ty)),
        Position::Static(x, y) => (Some(*x), Some(*y)),
    })
}

/// Translate the given shape into a new window (w, h) size to be applied to the window but takes
/// no direction action on the window. Window should already be unmaximized before calling this.
///
/// ### Arguments
/// * `w` - Window's current width
/// * `h` - Window's current height
/// * `bw` - Window's border width
/// * `bh` - Window's border height
/// * `aw` - Window manager's work area width
/// * `ah` - Window manager's work area height
/// * `pos` - Position to translate
///
/// ### Returns
/// * `(g, w, h)` size, or (None, 0, 0) for maximize, or (None, None, None) for no change
fn translate_shape(
    w: u32, h: u32, bw: u32, bh: u32, aw: u32, ah: u32, shape: &Shape,
) -> WmCtlResult<(Option<u32>, Option<u32>, Option<u32>)> {
    Ok(match shape {
        Shape::Max => (None, Some(0), Some(0)),
        Shape::UnMax => (None, None, None),
        _ => {
            // Pre-calculations
            // * return values from this function should NOT include the border sizes
            let fw = aw - bw; // full width = total width - border
            let fh = ah - bh; // full height = total height - border
            let hw = (aw as f32 / 2.0) as u32 - bw; // half width = total width - border
            let hh = (ah as f32 / 2.0) as u32 - bh; // half height = total height - border

            let (g, w, h) = match shape {
                // Grow the existing dimensions by 1% until full size
                Shape::Grow => {
                    // Remove the border before calculations are done
                    let mut w = ((w - bw) as f32 * 1.01) as u32 + bw;
                    if w >= fw {
                        w = fw
                    }
                    let mut h = ((h - bh) as f32 * 1.01) as u32 + bh;
                    if h >= fh {
                        h = fh
                    }
                    (Some(WinGravity::Center.into()), Some(w), Some(h))
                },

                // Half width x full height
                Shape::Halfw => (None, Some(hw), Some(fh)),

                // Full width x half height
                Shape::Halfh => (None, Some(fw), Some(hh)),

                // Half width x half height
                Shape::Small => (None, Some(hw), Some(hh)),

                // 3/4 short side x 4x3 sized long size
                Shape::Medium => {
                    let (w, h) = if ah < aw {
                        let h = fh as f32 * 0.75;
                        ((h * 4.0 / 3.0) as u32, h as u32)
                    } else {
                        let w = fw as f32 * 0.75;
                        (w as u32, (w * 4.0 / 3.0) as u32)
                    };
                    (None, Some(w), Some(h))
                },

                // Full short side x 4x3 sized long size
                Shape::Large => {
                    let (w, h) = if ah < aw {
                        ((fh as f32 * 4.0 / 3.0) as u32, fh)
                    } else {
                        (fw, (fw as f32 * 4.0 / 3.0) as u32)
                    };
                    (None, Some(w), Some(h))
                },

                // Shrink the existing dimensions by 1% down to no smaller than 100x100
                Shape::Shrink => {
                    // Remove the border before calculations are done
                    let mut w = (w - bw) as f32 * 0.99;
                    if w < 100.0 {
                        w = 100.0
                    }
                    let mut h = (h - bh) as f32 * 0.99;
                    if h < 100.0 {
                        h = 100.0
                    }
                    (Some(WinGravity::Center.into()), Some(w as u32 + bw), Some(h as u32 + bh))
                },

                // Use the static size provided
                Shape::Static(w, h) => (None, Some(*w), Some(*h)),

                // Don't change anything by default
                _ => (None, None, None),
            };
            (g, w, h)
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_shape_halfw() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (g, _x, _y) =
            translate_shape(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Shape::Halfw)
                .unwrap();
        let hw = (aw / 2.0) as u32;
        let fh = (ah) as u32;
        assert_eq!(_x, Some(hw));
        assert_eq!(_y, Some(fh));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (g, _x, _y) =
            translate_shape(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Shape::Halfw)
                .unwrap();
        let hw = (aw / 2.0) as u32;
        let fh = (ah) as u32;
        assert_eq!(_x, Some(hw));
        assert_eq!(_y, Some(fh));
    }

    #[test]
    fn test_translate_pos_bottomcenter() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::BottomCenter)
                .unwrap();
        let cx = (aw / 2.0 - (w + bw) / 2.0) as u32;
        let ty = (ah - h - bh) as u32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(ty));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::BottomCenter)
                .unwrap();
        let cx = (aw / 2.0 - (w + bw) / 2.0) as u32;
        let ty = (ah - h - bh) as u32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(ty));
    }

    #[test]
    fn test_translate_pos_topcenter() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::TopCenter)
                .unwrap();
        let cx = (aw / 2.0 - (w + bw) / 2.0) as u32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(0));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::TopCenter)
                .unwrap();
        let cx = (aw / 2.0 - (w + bw) / 2.0) as u32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(0));
    }

    #[test]
    fn test_translate_pos_rightcenter() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::RightCenter)
                .unwrap();
        let lx = (aw - w - bw) as u32;
        let cy = (ah / 2.0 - (h + bh) / 2.0) as u32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(cy));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::RightCenter)
                .unwrap();
        let lx = (aw - w - bw) as u32;
        let cy = (ah / 2.0 - (h + bh) / 2.0) as u32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(cy));
    }

    #[test]
    fn test_translate_pos_leftcenter() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::LeftCenter)
                .unwrap();
        let cy = (ah / 2.0 - (h + bh) / 2.0) as u32;
        assert_eq!(x, Some(0));
        assert_eq!(y, Some(cy));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::LeftCenter)
                .unwrap();
        let cy = (ah / 2.0 - (h + bh) / 2.0) as u32;
        assert_eq!(x, Some(0));
        assert_eq!(y, Some(cy));
    }

    #[test]
    fn test_translate_pos_bottomright() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::BottomRight)
                .unwrap();
        let lx = (aw - w - bw) as u32;
        let ty = (ah - h - bh) as u32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(ty));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::BottomRight)
                .unwrap();
        let lx = (aw - w - bw) as u32;
        let ty = (ah - h - bh) as u32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(ty));
    }

    #[test]
    fn test_translate_pos_bottomleft() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::BottomLeft)
                .unwrap();
        let ty = (ah - h - bh) as u32;
        assert_eq!(x, Some(0));
        assert_eq!(y, Some(ty));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::BottomLeft)
                .unwrap();
        let ty = (ah - h - bh) as u32;
        assert_eq!(x, Some(0));
        assert_eq!(y, Some(ty));
    }

    #[test]
    fn test_translate_pos_topright() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::TopRight)
                .unwrap();
        let lx = (aw - w - bw) as u32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(0));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::TopRight)
                .unwrap();
        let lx = (aw - w - bw) as u32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(0));
    }

    #[test]
    fn test_translate_pos_topleft() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::TopLeft)
                .unwrap();
        assert_eq!(x, Some(0));
        assert_eq!(y, Some(0));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::TopLeft)
                .unwrap();
        assert_eq!(x, Some(0));
        assert_eq!(y, Some(0));
    }

    #[test]
    fn test_translate_pos_bottom() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Bottom)
                .unwrap();
        let ty = (ah - h - bh) as u32;
        assert_eq!(x, None);
        assert_eq!(y, Some(ty));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Bottom)
                .unwrap();
        let ty = (ah - h - bh) as u32;
        assert_eq!(x, None);
        assert_eq!(y, Some(ty));
    }

    #[test]
    fn test_translate_pos_top() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Top).unwrap();
        assert_eq!(x, None);
        assert_eq!(y, Some(0));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Top).unwrap();
        assert_eq!(x, None);
        assert_eq!(y, Some(0));
    }

    #[test]
    fn test_translate_pos_right() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Right)
                .unwrap();
        let lx = (aw - w - bw) as u32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, None);

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Right)
                .unwrap();
        let lx = (aw - w - bw) as u32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, None);
    }

    #[test]
    fn test_translate_pos_left() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Left)
                .unwrap();
        assert_eq!(x, Some(0));
        assert_eq!(y, None);

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Left)
                .unwrap();
        assert_eq!(x, Some(0));
        assert_eq!(y, None);
    }

    #[test]
    fn test_translate_pos_center() {
        // No borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 0.0, 0.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Center)
                .unwrap();
        let cx = (aw / 2.0 - (w + bw) / 2.0) as u32;
        let cy = (ah / 2.0 - (h + bh) / 2.0) as u32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(cy));

        // With borders
        let (w, h, bw, bh, aw, ah) = (500.0, 500.0, 10.0, 10.0, 2560.0, 1415.0);
        let (x, y) =
            translate_pos(w as u32, h as u32, bw as u32, bh as u32, aw as u32, ah as u32, &Position::Center)
                .unwrap();
        let cx = (aw / 2.0 - (w + bw) / 2.0) as u32;
        let cy = (ah / 2.0 - (h + bh) / 2.0) as u32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(cy));
    }
}
