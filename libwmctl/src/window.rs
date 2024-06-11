use tracing::debug;

use crate::{model::*, WmCtlResult, WM};

/// Window provides a higer level interfacefor manipulating windows.
#[derive(Clone)]
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
        let id = id.unwrap_or(WM().read().unwrap().active_window().unwrap());
        Window::new(id)
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
        WM().read().unwrap().window_pid(self.id)
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
        WM().read().unwrap().window_name(self.id)
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
        WM().read().unwrap().window_class(self.id)
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
    pub fn kind(&self) -> WmCtlResult<Kind> {
        WM().read().unwrap().window_kind(self.id)
    }

    /// Get window state
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let state = win.state().unwrap();
    /// ```
    pub fn state(&self) -> WmCtlResult<Vec<State>> {
        WM().read().unwrap().window_state(self.id)
    }

    /// Get window parent
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let parent = win.parent().unwrap();
    /// ```
    pub fn parent(&self) -> WmCtlResult<Window> {
        WM().read().unwrap().window_parent(self.id)
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
        WM().read().unwrap().window_desktop(self.id)
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
        WM().read().unwrap().window_geometry(self.id)
    }

    /// Get visual window geometry
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let (x, y, w, h) = win.visual_geometry().unwrap();
    /// ```
    pub fn visual_geometry(&self) -> WmCtlResult<(i32, i32, u32, u32)> {
        WM().read().unwrap().window_visual_geometry(self.id)
    }

    /// Get window frame border values added by the window manager
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let (l, r, t, b) = win.borders().unwrap();
    /// ```
    pub fn borders(&self) -> WmCtlResult<Border> {
        WM().read().unwrap().window_borders(self.id)
    }

    /// Determine if this window is a GTK application
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let result = win.is_gtk();
    /// ```
    pub fn is_gtk(&self) -> bool {
        WM().read().unwrap().window_is_gtk(self.id)
    }

    /// Get window GNOME border values added by GTK
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let (l, r, t, b) = win.gtk_borders().unwrap();
    /// ```
    pub fn gtk_borders(&self) -> WmCtlResult<Border> {
        WM().read().unwrap().window_gtk_borders(self.id)
    }

    /// Get window mapped state
    /// * doesn't return a valid state if all windows are included rather than just the managed ones
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// let state = win.mapped().unwrap();
    /// ```
    pub fn mapped(&self) -> WmCtlResult<MapState> {
        WM().read().unwrap().window_attributes(self.id)
    }

    /// Get all window properties generically
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// win.properties().unwrap();
    /// ```
    pub fn properties(&self) -> WmCtlResult<Vec<Property>> {
        WM().read().unwrap().window_properties(self.id)
    }

    /// Map the window to the screen
    /// * Windows are created in the unmapped state and must be mapped to be visible
    /// * Unmapping the window will have the opposite effect of hidding the window
    /// * Useful for new windows or dialogs that need to conditionally be visible
    /// * It is much faster to hide and show and window rather than recreate it
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = window(12345);
    /// win.map().unwrap();
    /// ```
    pub fn map(&self) -> WmCtlResult<()> {
        WM().read().unwrap().map_window(self.id)
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
        WM().read().unwrap().maximize_window(self.id)
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
        self.state().is_ok_and(|states| states.contains(&State::MaxVert) || states.contains(&State::MaxHorz))
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
        WM().read().unwrap().unmaximize_window(self.id)
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

    /// Queue the position the window should be. This will not take effect until the place() method
    /// is called.
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
        let wm = WM().read().unwrap();

        // Unmaximize to shape and position the window correctly
        if self.maximized() {
            self.unmaximize()?;
        }

        // Get window properties
        let border = self.borders()?;
        let csd_border = self.gtk_borders()?;
        let (_, _, w, h) = self.geometry()?;
        let size = Rect::new(w, h);
        let area = Rect::new(wm.work_width, wm.work_height);

        // Shape the window as directed
        let (gravity, sw, sh) = if let Some(shape) = self.shape.as_ref() {
            let (gravity, sw, sh) = translate_shape(&size, &border, &csd_border, &area, shape)?;

            // Don't use gravity if positioning is required
            if self.pos.is_some() {
                (None, sw, sh)
            } else {
                (gravity, sw, sh)
            }
        } else {
            (None, None, None)
        };

        // Position the window if directed
        let (x, y) = if let Some(pos) = &self.pos {
            translate_pos(&size, &border, &csd_border, &area, pos)?
        } else {
            (None, None)
        };

        // Execute if reason to
        debug!("place: {:?}, {:?}, {}, {}", x, y, w, h);
        wm.move_resize_window(self.id, gravity, x, y, sw, sh)
    }
}

/// Translate position enum values into (x, y) cordinates but takes no direct action on the window.
/// Window should already be unmaximized before calling this function.
///
/// ### Arguments
/// * `size` - Window's current width and height
/// * `border` - Window's border left, right, top, and bottom
/// * `csd_border` - Client side border left, right, top, and bottom
/// * `area` - Window manager's work area width and height
/// * `pos` - Position to translate
///
/// ### Returns
/// * `(x, y)` cordinates or (None, None) for no change
fn translate_pos(
    size: &Rect, border: &Border, csd_border: &Border, area: &Rect, pos: &Position,
) -> WmCtlResult<(Option<i32>, Option<i32>)> {
    // Pre-calculating some commonly used values for the translation
    let csd = csd_border.any();

    // Centering algorithm: wether the window has CSD borders part of the app or are added on
    // after the fact by the window manager the algorithm at its root is the same. We need to
    // ensure the borders are subtracted and calculated separately as they are frequently not
    // the same size.

    // left x coordinate of window such that the window will appear horizontally centered.
    let cx = if csd {
        let offset = csd_border.w() as f32 / 2.0;
        ((area.w as f32 - size.w as f32 - csd_border.w() as f32) / 2.0 + offset) as i32
    } else {
        ((area.w as f32 - (size.w as f32 + border.w() as f32)) / 2.0) as i32
    };

    // top y coordinate of window such that the window will appear vertically centered
    let cy = if csd {
        let offset = csd_border.h() as f32 / 2.0;
        ((area.h as f32 - size.h as f32 - csd_border.h() as f32) / 2.0 + offset) as i32
    } else {
        ((area.h as f32 - (size.h as f32 + border.h() as f32)) / 2.0) as i32
    };

    // left x coordinate for the window such that the window will appear on the right
    let lxr = if csd {
        area.w as i32 - size.w as i32 + csd_border.r as i32
    } else {
        area.w as i32 - size.w as i32 - border.w() as i32
    };

    // left x coordinate for the window such that the window will appear on the left
    let lxl = if csd { 0 - csd_border.l as i32 } else { 0 };

    // top y coordinate for the window such that the window will appear all the way to the top
    let ty = if csd { 0 - csd_border.t as i32 } else { 0 };

    // bottom y coordinate for the window such that the window will appear at the bottom
    let by = if csd {
        area.h as i32 - size.h as i32 + csd_border.b as i32
    } else {
        area.h as i32 - size.h as i32 - border.h() as i32
    };

    Ok(match pos {
        Position::Center => (Some(cx), Some(cy)),
        Position::Left => (Some(lxl), None),
        Position::Right => (Some(lxr), None),
        Position::Top => (None, Some(ty)),
        Position::Bottom => (None, Some(by)),
        Position::TopLeft => (Some(lxl), Some(ty)),
        Position::TopRight => (Some(lxr), Some(ty)),
        Position::BottomLeft => (Some(lxl), Some(by)),
        Position::BottomRight => (Some(lxr), Some(by)),
        Position::LeftCenter => (Some(lxl), Some(cy)),
        Position::RightCenter => (Some(lxr), Some(cy)),
        Position::TopCenter => (Some(cx), Some(ty)),
        Position::BottomCenter => (Some(cx), Some(by)),
        Position::Static(x, y) => {
            if csd {
                (Some(*x - csd_border.l as i32), Some(*y - csd_border.t as i32))
            } else {
                (Some(*x), Some(*y))
            }
        },
    })
}

/// Translate the given shape into a new window (w, h) size to be applied to the window but takes
/// no direction action on the window. Window should already be unmaximized before calling this.
///
/// ### Arguments
/// * `size` - Window's current (width, height)
/// * `border` - Window Manager's borders left, right, top, bottom
/// * `csd_border` - Client side decorations left, right, top, bottom
/// * `area` - Window manager's work area (width, height)
/// * `shape` - Desired shape to make the window
///
/// ### Returns
/// * `(g, w, h)` size, or (None, 0, 0) for maximize, or (None, None, None) for no change
fn translate_shape(
    size: &Rect, border: &Border, csd_border: &Border, area: &Rect, shape: &Shape,
) -> WmCtlResult<(Option<u32>, Option<u32>, Option<u32>)> {
    Ok(match shape {
        Shape::Max => (None, Some(0), Some(0)),
        Shape::UnMax => (None, None, None),
        _ => {
            // Determine if the window has CSD borders
            let csd = csd_border.any();

            // Pre-calculations
            // * return values from this function should NOT include the border sizes for regular
            //   windows as the Window Manager will calculate the border size for the window.
            // * return values from this function should include the border sizes for CSD windows
            //   as the Window Manager doesn't know about the client side decorations.

            // full width = total width + border || total width - border
            let fw = if csd { area.w + csd_border.w() } else { area.w - border.w() };

            // full height = total height + CSD border || total height - border
            let fh = if csd { area.h + csd_border.h() } else { area.h - border.h() };

            // half width = total width + CSD border || total width - border
            let hw = if csd {
                (area.w as f32 / 2.0) as u32 + csd_border.w()
            } else {
                (area.w as f32 / 2.0) as u32 - border.w()
            };

            // half height = total height + CSD border || total height - border
            let hh = if csd {
                (area.h as f32 / 2.0) as u32 + csd_border.h()
            } else {
                (area.h as f32 / 2.0) as u32 - border.h()
            };

            let (g, w, h) = match shape {
                // Grow the existing dimensions by 1% until full size
                // * Caculate with CSD borders for client side decorations
                // * Caculate without borders for regular windows
                Shape::Grow => {
                    let mut w = if csd {
                        ((size.w + csd_border.w()) as f32 * 1.01) as u32
                    } else {
                        ((size.w - border.w()) as f32 * 1.01) as u32
                    };
                    if w >= fw {
                        w = fw
                    }
                    let mut h = if csd {
                        ((size.h + csd_border.h()) as f32 * 1.01) as u32
                    } else {
                        ((size.h - border.h()) as f32 * 1.01) as u32
                    };
                    if h >= fh {
                        h = fh
                    }

                    // Use center gravity to grow the window in all directions
                    (Some(Gravity::Center.into()), Some(w), Some(h))
                },

                // Half width x full height
                Shape::Halfw => (None, Some(hw), Some(fh)),

                // Full width x half height
                Shape::Halfh => (None, Some(fw), Some(hh)),

                // Half width x half height
                Shape::Small => (None, Some(hw), Some(hh)),

                // 3/4 short side x 4x3 sized long size
                Shape::Medium => {
                    let (w, h) = if area.h < area.w {
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
                    let (w, h) = if area.h < area.w {
                        ((fh as f32 * 4.0 / 3.0) as u32, fh)
                    } else {
                        (fw, (fw as f32 * 4.0 / 3.0) as u32)
                    };
                    (None, Some(w), Some(h))
                },

                // Shrink the existing dimensions by 1% down to no smaller than 100x100
                // * Caculate with CSD borders for client side decorations
                // * Caculate without borders for regular windows
                Shape::Shrink => {
                    let mut w = if csd {
                        (size.w + csd_border.w()) as f32 * 0.99
                    } else {
                        (size.w - border.w()) as f32 * 0.99
                    };
                    if w < 100.0 {
                        w = 100.0
                    }
                    let mut h = if csd {
                        (size.h + csd_border.h()) as f32 * 0.99
                    } else {
                        (size.h - border.h()) as f32 * 0.99
                    };
                    if h < 100.0 {
                        h = 100.0
                    }

                    // Use center gravity to shrink the window in all directions
                    (Some(Gravity::Center.into()), Some(w as u32), Some(h as u32))
                },

                // Use the static size provided
                // * Include borders for client side decorations
                // * Don't include borders for regular windows
                Shape::Static(w, h) => {
                    if csd {
                        (None, Some(*w + csd_border.w()), Some(*h + csd_border.h()))
                    } else {
                        (None, Some(*w), Some(*h))
                    }
                },

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
        let size = Rect::default();
        let borders = Border::default();
        let csd = Border::default();
        let area = Rect { w: 2560, h: 1415 };
        let (g, _w, _h) = translate_shape(&size, &borders, &csd, &area, &Shape::Halfw).unwrap();
        let hw = (area.w as f32 / 2.0) as u32;
        let fh = (area.h) as u32;
        assert_eq!(g, None);
        assert_eq!(_w, Some(hw));
        assert_eq!(_h, Some(fh));

        // With window manager borders
        let borders = Border::new(5, 5, 10, 10);
        let area = Rect { w: 2560, h: 1415 };
        let (g, _w, _h) = translate_shape(&size, &borders, &csd, &area, &Shape::Halfw).unwrap();
        let hw = (area.w as f32 / 2.0) as u32 - borders.w();
        let fh = (area.h) as u32 - borders.h();
        assert_eq!(g, None);
        assert_eq!(_w, Some(hw));
        assert_eq!(_h, Some(fh));

        // With csd borders
        let csd = Border::new(5, 5, 10, 10);
        let area = Rect { w: 2560, h: 1415 };
        let (g, _w, _h) = translate_shape(&size, &borders, &csd, &area, &Shape::Halfw).unwrap();
        let hw = (area.w as f32 / 2.0) as u32 + csd.w();
        let fh = (area.h) as u32 + csd.h();
        assert_eq!(g, None);
        assert_eq!(_w, Some(hw));
        assert_eq!(_h, Some(fh));
    }

    #[test]
    fn test_translate_pos_bottomcenter() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::BottomCenter,
        )
        .unwrap();
        let cx = ((aw as f32 - w as f32) / 2.0) as i32;
        let by = (ah - h) as i32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(by));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::BottomCenter,
        )
        .unwrap();
        let cx = ((aw as f32 - (w as f32 + b.w() as f32)) / 2.0) as i32;
        let by = (ah as f32 - h as f32 - b.h() as f32) as i32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(by));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::BottomCenter,
        )
        .unwrap();
        let offset = c.w() as f32 / 2.0;
        let cx = ((aw as f32 - w as f32 - c.w() as f32) / 2.0 + offset) as i32;
        let by = (ah as f32 - h as f32 + c.b as f32) as i32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(by));
    }

    #[test]
    fn test_translate_pos_topcenter() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::TopCenter,
        )
        .unwrap();
        let cx = ((aw as f32 - w as f32) / 2.0) as i32;
        let ty = 0;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(ty));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::TopCenter,
        )
        .unwrap();
        let cx = ((aw as f32 - (w as f32 + b.w() as f32)) / 2.0) as i32;
        let ty = 0;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(ty));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::TopCenter,
        )
        .unwrap();
        let offset = c.w() as f32 / 2.0;
        let cx = ((aw as f32 - w as f32 - c.w() as f32) / 2.0 + offset) as i32;
        let by = 0 - c.t as i32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(by));
    }

    #[test]
    fn test_translate_pos_rightcenter() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::RightCenter,
        )
        .unwrap();
        let rx = aw as i32 - w as i32;
        let cy = ((ah as f32 - h as f32) / 2.0) as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, Some(cy));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::RightCenter,
        )
        .unwrap();
        let rx = aw as i32 - w as i32 - b.w() as i32;
        let cy = ((ah as f32 - (h as f32 + b.h() as f32)) / 2.0) as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, Some(cy));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::RightCenter,
        )
        .unwrap();
        let rx = aw as i32 - w as i32 + b.r as i32;
        let offset = c.h() as f32 / 2.0;
        let cy = ((ah as f32 - h as f32 - c.h() as f32) / 2.0 + offset) as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, Some(cy));
    }

    #[test]
    fn test_translate_pos_leftcenter() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::LeftCenter,
        )
        .unwrap();
        let lx = 0;
        let cy = ((ah as f32 - h as f32) / 2.0) as i32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(cy));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::LeftCenter,
        )
        .unwrap();
        let lx = 0;
        let cy = ((ah as f32 - (h as f32 + b.h() as f32)) / 2.0) as i32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(cy));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::LeftCenter,
        )
        .unwrap();
        let lx = 0 - b.l as i32;
        let offset = c.h() as f32 / 2.0;
        let cy = ((ah as f32 - h as f32 - c.h() as f32) / 2.0 + offset) as i32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(cy));
    }

    #[test]
    fn test_translate_pos_bottomright() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::BottomRight,
        )
        .unwrap();
        let rx = aw as i32 - w as i32;
        let by = (ah - h) as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, Some(by));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::BottomRight,
        )
        .unwrap();
        let rx = aw as i32 - w as i32 - b.w() as i32;
        let by = (ah as f32 - h as f32 - b.h() as f32) as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, Some(by));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::BottomRight,
        )
        .unwrap();
        let rx = aw as i32 - w as i32 + b.r as i32;
        let by = (ah as f32 - h as f32 + c.b as f32) as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, Some(by));
    }

    #[test]
    fn test_translate_pos_bottomleft() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::BottomLeft,
        )
        .unwrap();
        let lx = 0;
        let by = (ah - h) as i32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(by));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::BottomLeft,
        )
        .unwrap();
        let lx = 0;
        let by = (ah as f32 - h as f32 - b.h() as f32) as i32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(by));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::BottomLeft,
        )
        .unwrap();
        let lx = 0 - b.l as i32;
        let by = (ah as f32 - h as f32 + c.b as f32) as i32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(by));
    }

    #[test]
    fn test_translate_pos_topright() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::TopRight,
        )
        .unwrap();
        let rx = aw as i32 - w as i32;
        let ty = 0;
        assert_eq!(x, Some(rx));
        assert_eq!(y, Some(ty));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::TopRight,
        )
        .unwrap();
        let rx = aw as i32 - w as i32 - b.w() as i32;
        let ty = 0;
        assert_eq!(x, Some(rx));
        assert_eq!(y, Some(ty));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::TopRight,
        )
        .unwrap();
        let rx = aw as i32 - w as i32 + b.r as i32;
        let ty = 0 - c.t as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, Some(ty));
    }

    #[test]
    fn test_translate_pos_topleft() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::TopLeft,
        )
        .unwrap();
        let lx = 0;
        let ty = 0;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(ty));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::TopLeft,
        )
        .unwrap();
        let lx = 0;
        let ty = 0;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(ty));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::TopLeft,
        )
        .unwrap();
        let lx = 0 - b.l as i32;
        let ty = 0 - c.t as i32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, Some(ty));
    }

    #[test]
    fn test_translate_pos_bottom() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Bottom,
        )
        .unwrap();
        let by = (ah - h) as i32;
        assert_eq!(x, None);
        assert_eq!(y, Some(by));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Bottom,
        )
        .unwrap();
        let by = (ah as f32 - h as f32 - b.h() as f32) as i32;
        assert_eq!(x, None);
        assert_eq!(y, Some(by));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::Bottom,
        )
        .unwrap();
        let by = (ah as f32 - h as f32 + c.b as f32) as i32;
        assert_eq!(x, None);
        assert_eq!(y, Some(by));
    }

    #[test]
    fn test_translate_pos_top() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Top,
        )
        .unwrap();
        let ty = 0;
        assert_eq!(x, None);
        assert_eq!(y, Some(ty));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Top,
        )
        .unwrap();
        let ty = 0;
        assert_eq!(x, None);
        assert_eq!(y, Some(ty));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::Top,
        )
        .unwrap();
        let ty = 0 - c.t as i32;
        assert_eq!(x, None);
        assert_eq!(y, Some(ty));
    }

    #[test]
    fn test_translate_pos_right() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Right,
        )
        .unwrap();
        let rx = aw as i32 - w as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, None);

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Right,
        )
        .unwrap();
        let rx = aw as i32 - w as i32 - b.w() as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, None);

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::Right,
        )
        .unwrap();
        let rx = aw as i32 - w as i32 + b.r as i32;
        assert_eq!(x, Some(rx));
        assert_eq!(y, None);
    }

    #[test]
    fn test_translate_pos_left() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Left,
        )
        .unwrap();
        let lx = 0;
        assert_eq!(x, Some(lx));
        assert_eq!(y, None);

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Left,
        )
        .unwrap();
        let lx = 0;
        assert_eq!(x, Some(lx));
        assert_eq!(y, None);

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::Left,
        )
        .unwrap();
        let lx = 0 - b.l as i32;
        assert_eq!(x, Some(lx));
        assert_eq!(y, None);
    }

    #[test]
    fn test_translate_pos_center() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Center,
        )
        .unwrap();
        let cx = ((aw as f32 - w as f32) / 2.0) as i32;
        let cy = ((ah as f32 - h as f32) / 2.0) as i32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(cy));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Center,
        )
        .unwrap();
        let cx = ((aw as f32 - (w as f32 + b.w() as f32)) / 2.0) as i32;
        let cy = ((ah as f32 - (h as f32 + b.h() as f32)) / 2.0) as i32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(cy));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::Center,
        )
        .unwrap();
        let x_offset = c.w() as f32 / 2.0;
        let cx = ((aw as f32 - w as f32 - c.w() as f32) / 2.0 + x_offset) as i32;
        let y_offset = c.h() as f32 / 2.0;
        let cy = ((ah as f32 - h as f32 - c.h() as f32) / 2.0 + y_offset) as i32;
        assert_eq!(x, Some(cx));
        assert_eq!(y, Some(cy));
    }

    #[test]
    fn test_translate_pos_static() {
        // No borders
        let (aw, ah) = (2560.0, 1415.0);
        let (w, h) = (500.0, 500.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Static(10, 10),
        )
        .unwrap();
        assert_eq!(x, Some(10));
        assert_eq!(y, Some(10));

        // With WM borders
        let b = Border::new(10, 10, 10, 10);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &b,
            &Border::default(),
            &Rect::new(aw as u32, ah as u32),
            &Position::Static(10, 10),
        )
        .unwrap();
        assert_eq!(x, Some(10));
        assert_eq!(y, Some(10));

        // With CSD borders
        let c = Border::new(10, 10, 10, 10);
        let (w, h) = (480.0, 480.0);
        let (x, y) = translate_pos(
            &Rect::new(w as u32, h as u32),
            &Border::default(),
            &c,
            &Rect::new(aw as u32, ah as u32),
            &Position::Static(10, 10),
        )
        .unwrap();
        assert_eq!(x, Some(0));
        assert_eq!(y, Some(0));
    }
}
