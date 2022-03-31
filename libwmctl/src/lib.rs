//! `libwmctl` implements the [Extended Window Manager Hints (EWMH) specification](https://specifications.freedesktop.org/wm-spec/latest/)
//! as a way to integrate with EWMH compatible window managers. The EWHM spec builds on the lower
//! level Inter Client Communication Conventions Manual (ICCCM) to define interactions between
//! window managers, compositing managers and applications.
//!
//! [Root Window Properties](https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html)  
//! The EWMH spec defines a number of properties that EWHM compliant window managers will maintain
//! and return to clients requesting information. `libwmctl` taps into the message queue to retrieve
//! details about a given window and to than manipulate the given window as desired.
//!
//! `wmctl` uses `libwmctl` with pre-defined shapes and positions to manipulate how a window should
//! be shaped and positioned on the screen in an ergonomic way; however `libwmctl` could be used
//! for a variety of reasons.
mod error;
mod model;
mod wmctl;
pub use error::*;
pub use model::*;
pub use wmctl::WmCtl;

/// All essential symbols in a simple consumable form
///
/// ### Examples
/// ```
/// use libwmctl::prelude::*;
/// ```
pub mod prelude
{
    pub use crate::*;
}

/// Window option provides an ergonomic way to manipulate a window
pub struct WinOpt
{
    win: Option<u32>,
    w: Option<u32>,
    h: Option<u32>,
    x: Option<u32>,
    y: Option<u32>,
    shape: Option<WinShape>,
    pos: Option<WinPosition>,
}

impl WinOpt
{
    /// Create a new window option with the given optional window.
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate else the active window will be used
    ///
    /// ### Examples
    /// ```
    /// use libwmctl::prelude::*;
    /// let win = WinOpt::new(None);
    /// ```
    pub fn new(win: Option<u32>) -> Self
    {
        Self {
            win,
            w: Default::default(),
            h: Default::default(),
            x: Default::default(),
            y: Default::default(),
            shape: Default::default(),
            pos: Default::default(),
        }
    }

    /// Set the width and height the window should be. This option takes priority over
    /// and will set the shape option to None.
    ///
    /// ### Arguments
    /// * `w` - width the window should be
    /// * `h` - height the window should be
    ///
    /// ### Examples
    /// ```
    /// use libwmctl::prelude::*;
    /// let win = WinOpt::new(None).size(500, 500);
    /// ```
    pub fn size(mut self, w: u32, h: u32) -> Self
    {
        self.w = Some(w);
        self.h = Some(h);
        self.shape = None;
        self
    }

    /// Set the x, y location the window should be. This option takes priority over
    /// and will set the position option to None.
    ///
    /// ### Arguments
    /// * `x` - x coordinate the window moved to
    /// * `y` - y coordinate the window moved to
    ///
    /// ### Examples
    /// ```
    /// use libwmctl::prelude::*;
    /// let win = WinOpt::new(None).location(0, 0);
    /// ```
    pub fn location(mut self, x: u32, y: u32) -> Self
    {
        self.x = Some(x);
        self.y = Some(y);
        self.pos = None;
        self
    }

    /// Set the shape the window should be. This option will not be set unless
    /// the width and height options are None.
    ///
    /// ### Arguments
    /// * `shape` - pre-defined shape to manipulate the window into
    ///
    /// ### Examples
    /// ```
    /// use libwmctl::prelude::*;
    /// let win = WinOpt::new(None).shape(WinShape::Large);
    /// ```
    pub fn shape(mut self, shape: WinShape) -> Self
    {
        if self.w.is_none() && self.h.is_none() {
            self.shape = Some(shape);
        }
        self
    }

    /// Set the position the window should be. This option will not be set unless
    /// the x and y opitons are None.
    ///
    /// ### Arguments
    /// * `pos` - pre-defined position to move the window to
    ///
    /// ### Examples
    /// ```
    /// use libwmctl::prelude::*;
    /// let win = WinOpt::new(None).pos(WinPosition::Right);
    /// ```
    pub fn pos(mut self, pos: WinPosition) -> Self
    {
        if self.x.is_none() && self.y.is_none() {
            self.pos = Some(pos);
        }
        self
    }

    // Check if any options are set
    fn any(&self) -> bool
    {
        self.w.is_some()
            || self.h.is_some()
            || self.x.is_some()
            || self.y.is_some()
            || self.shape.is_some()
            || self.pos.is_some()
    }

    /// Place the window according to the specified options
    ///
    /// ### Examples
    /// ```ignore
    /// use libwmctl::prelude::*;
    /// let win = WinOpt::new(None).shape(WinShape::Large).pos(WinPosition::Right);
    /// ```
    pub fn place(self) -> WmCtlResult<()>
    {
        let execute = self.any();
        let wmctl = WmCtl::connect()?;

        // Get window properties
        let win = self.win.unwrap_or(wmctl.active_win()?);
        let (bl, br, bt, bb) = wmctl.win_borders(win)?;
        let (_, _, w, h) = wmctl.win_geometry(win)?;

        // Shape the window as directed
        let (gravity, sw, sh) = if let Some(shape) = self.shape {
            let (gravity, sw, sh) = shape_win(&wmctl, win, w, h, bl + br, bt + bb, shape)?;

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
        let (x, y) = if let Some(pos) = self.pos {
            move_win(&wmctl, win, sw.unwrap_or(w), sh.unwrap_or(h), bl + br, bt + bb, pos)?
        } else if self.x.is_some() && self.y.is_some() {
            (self.x, self.y)
        } else {
            (None, None)
        };

        // Execute if reason to
        if execute {
            wmctl.move_resize_win(win, gravity, x, y, sw, sh)
        } else {
            Ok(())
        }
    }
}

/// List out a x11 information including the window manager's name, if there is a composite
/// manager running and what the screen and work screen sizes are.
///
/// ### Arguments
/// * `win` - id of the window to manipulate else the active window will be used
///
/// ### Examples
/// ```ignore
/// use libwmctl::prelude::*;
/// libwmctl::info(None).unwrap();
/// ```
pub fn info(win: Option<u32>) -> WmCtlResult<()>
{
    let wmctl = WmCtl::connect()?;
    let (_, wm_name) = wmctl.winmgr()?;
    let win = win.unwrap_or(wmctl.active_win()?);
    println!("X11 Information");
    println!("-----------------------------------------------------------------------");
    println!("Window Manager:    {}", wm_name);
    println!("Composite Manager: {}", wmctl.composite_manager()?);
    println!("Root Window:       {}", wmctl.root);
    println!("Work area:         {}x{}", wmctl.work_width, wmctl.work_height);
    println!("Screen Size:       {}x{}", wmctl.width, wmctl.height);
    println!("Desktops:          {}", wmctl.desktops()?);
    println!();
    println!("Active Window");
    println!("{:-<120}", "");
    print_win_header();
    print_win_details(&wmctl, win)?;
    wmctl.win_attributes(win)?;
    Ok(())
}

/// List out the windows the window manager is managing and their essential properties
///
/// ### Arguments
/// * `all` - when set to true will list all x11 windows not just those the window manager lists
///
/// ### Examples
/// ```ignore
/// use libwmctl::prelude::*;
/// libwmctl::list().unwrap();
/// ```
pub fn list(all: bool) -> WmCtlResult<()>
{
    let wmctl = WmCtl::connect()?;
    print_win_header();
    for win in wmctl.windows(all)? {
        print_win_details(&wmctl, win)?;
    }
    Ok(())
}

fn print_win_header()
{
    println!(
        "{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}",
        "ID", "DSK", "PID", "X", "Y", "W", "H", "BORDERS", "TYPE", "STATE", "CLASS", "NAME"
    );
}

fn print_win_details(wmctl: &WmCtl, win: u32) -> WmCtlResult<()>
{
    let pid = wmctl.win_pid(win).unwrap_or(-1);
    let desktop = wmctl.win_desktop(win).unwrap_or(-1);
    let typ = wmctl.win_type(win).unwrap_or(WinType::Invalid);
    let states = wmctl.win_state(win).unwrap_or(vec![WinState::Invalid]);
    let (x, y, w, h) = wmctl.win_geometry(win)?;
    let (l, r, t, b) = wmctl.win_borders(win).unwrap_or((0, 0, 0, 0));
    let class = wmctl.win_class(win).unwrap_or("".to_owned());
    let name = wmctl.win_name(win).unwrap_or("".to_owned());
    println!(
        "{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}",
        format!("{:0>8}", win),
        format!("{:>2}", desktop),
        pid,
        format!("{:<4}", x),
        format!("{:<4}", y),
        format!("{:<4}", w),
        format!("{:<4}", h),
        format!("{},{},{},{}", l, r, t, b),
        typ.to_string(),
        format!("{:?}", states),
        class,
        name
    );
    Ok(())
}

/// Move the given window or active window if not given without changing its size
fn move_win(
    wmctl: &WmCtl, win: u32, w: u32, h: u32, bw: u32, bh: u32, pos: WinPosition,
) -> WmCtlResult<(Option<u32>, Option<u32>)>
{
    wmctl.unmaximize_win(win)?;

    // Pre-calculations
    let cx = if (w + bw) / 2 >= wmctl.work_width / 2 { 0 } else { wmctl.work_width / 2 - (w + bw) / 2 }; // center x
    let cy = if (h + bh) / 2 >= wmctl.work_height / 2 { 0 } else { wmctl.work_height / 2 - (h + bh) / 2 }; // center y
    let lx = if w + bw >= wmctl.work_width { 0 } else { wmctl.work_width - w - bw }; // left x
    let ty = if h + bh >= wmctl.work_height { 0 } else { wmctl.work_height - h - bh }; // top y

    // Interpret the position as x, y cordinates
    Ok(match pos {
        WinPosition::Center => (Some(cx), Some(cy)),
        WinPosition::Left => (Some(0), None),
        WinPosition::Right => (Some(lx), None),
        WinPosition::Top => (None, Some(0)),
        WinPosition::Bottom => (None, Some(ty)),
        WinPosition::TopLeft => (Some(0), Some(0)),
        WinPosition::TopRight => (Some(lx), Some(0)),
        WinPosition::BottomLeft => (Some(0), Some(ty)),
        WinPosition::BottomRight => (Some(lx), Some(ty)),
        WinPosition::LeftCenter => (Some(0), Some(cy)),
        WinPosition::RightCenter => (Some(lx), Some(cy)),
        WinPosition::TopCenter => (Some(cx), Some(0)),
        WinPosition::BottomCenter => (Some(cx), Some(ty)),
    })
}

/// Shape the given window or active window if not given without moving it.
fn shape_win(
    wmctl: &WmCtl, win: u32, w: u32, h: u32, bw: u32, bh: u32, shape: WinShape,
) -> WmCtlResult<(Option<u32>, Option<u32>, Option<u32>)>
{
    // Notes
    // * return values from this func should not include the border sizes
    Ok(match shape {
        WinShape::Max => {
            wmctl.maximize_win(win)?;
            (None, None, None)
        },
        WinShape::UnMax => {
            wmctl.unmaximize_win(win)?;
            (None, None, None)
        },
        _ => {
            wmctl.unmaximize_win(win)?;

            // Pre-calculations
            let fw = wmctl.work_width - bw; // total width - border
            let fh = wmctl.work_height - bh; // total height - border
            let hw = wmctl.work_width / 2 - bw; // total half width - border
            let hh = wmctl.work_height / 2 - bh; // total half height - border

            let (w, h) = match shape {
                // Grow the existing dimensions by 1% until full size
                WinShape::Grow => {
                    let mut w = ((w - bw) as f32 * 1.01) as u32 + bw;
                    if w >= fw {
                        w = fw
                    }
                    let mut h = ((h - bh) as f32 * 1.01) as u32 + bh;
                    if h >= fh {
                        h = fh
                    }
                    (Some(w), Some(h))
                },

                // Half width x full height
                WinShape::Halfw => (Some(hw), Some(fh)),

                // Full width x half height
                WinShape::Halfh => (Some(fw), Some(hh)),

                // Half width x half height
                WinShape::Small => (Some(hw), Some(hh)),

                // 3/4 short side x 4x3 sized long size
                WinShape::Medium => {
                    let (w, h) = if wmctl.work_height < wmctl.work_width {
                        let h = fh as f32 * 0.75;
                        ((h * 4.0 / 3.0) as u32, h as u32)
                    } else {
                        let w = fw as f32 * 0.75;
                        (w as u32, (w * 4.0 / 3.0) as u32)
                    };
                    (Some(w), Some(h))
                },

                // Full short side x 4x3 sized long size
                WinShape::Large => {
                    let (w, h) = if wmctl.work_height < wmctl.work_width {
                        ((fh as f32 * 4.0 / 3.0) as u32, fh)
                    } else {
                        (fw, (fw as f32 * 4.0 / 3.0) as u32)
                    };
                    (Some(w), Some(h))
                },

                // Shrink the existing dimensions by 1% down to no smaller than 100x100
                WinShape::Shrink => {
                    // Remove the border before calculations are done then re-include
                    let mut w = (w - bw) as f32 * 0.99;
                    if w < 100.0 {
                        w = 100.0
                    }
                    let mut h = (h - bh) as f32 * 0.99;
                    if h < 100.0 {
                        h = 100.0
                    }
                    (Some(w as u32 + bw), Some(h as u32 + bh))
                },

                // Don't change anything by default
                _ => (None, None),
            };
            (Some(WinGravity::Center.into()), w, h)
        },
    })
}

#[cfg(test)]
mod tests
{
    #[test]
    fn it_works()
    {
        assert_eq!(2 + 2, 4);
    }
}
