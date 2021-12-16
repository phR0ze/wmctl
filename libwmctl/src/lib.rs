mod wmctl;
mod error;
mod model;
use wmctl::*;
use error::*;
use model::*;
use tracing::trace;

/// All essential symbols in a simple consumable form
///
/// ### Examples
/// ```
/// use libwmctl::prelude::*;
/// ```
pub mod prelude {
    pub use crate::*;
    pub use error::*;
    pub use model::*;
}

/// Get x11 info
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
    Ok(())
}

/// List out all the current window ids and their details
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
    println!("{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}", "ID", "DSK", "PID", "X", "Y", "W", "H", "BORDERS", "TYPE", "STATE", "CLASS", "NAME");
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
    println!("{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}",
        format!("{:0>8}", win), format!("{:>2}", desktop), pid,
        format!("{:<4}", x), format!("{:<4}", y), format!("{:<4}", w), format!("{:<4}", h), 
        format!("{},{},{},{}", l, r, t, b),
        typ.to_string(), format!("{:?}", states), class, name);
    Ok(())
}

/// Place the window by optionally modifying its size and optionally modifying its position
pub fn place(win: Option<u32>, shape: Option<WinShape>, pos: Option<WinPosition>) -> WmCtlResult<()>
{
    let wmctl = WmCtl::connect()?;

    // Get the window properties
    let win = win.unwrap_or(wmctl.active_win()?);
    let (_, _, w, h) = wmctl.win_geometry(win)?;
    let (bl, br, bt, bb) = wmctl.win_borders(win)?;

    // Shape the window if directed
    let (gravity, sw, sh) = if let Some(shape) = shape {
        shape_win(&wmctl, win, w, h, bl + br, bt + bb, shape)?
    } else {
        (None, None, None)
    };

    // Don't use gravity if positioning is required
    let gravity = if pos.is_some() {
        None
    } else {
        gravity
    };

    // Position the window if directed
    let (x, y) = if let Some(pos) = pos {
        move_win(&wmctl, win, sw.unwrap_or(w), sh.unwrap_or(h), bl + br, bt + bb, pos)?
    } else {
        (None, None)
    };

    // Execute the shaping and positioning
    wmctl.move_resize_win(win, gravity, x, y, sw, sh)
}

/// Move the given window or active window if not given without changing its size
fn move_win(wmctl: &WmCtl, win: u32, w: u32, h: u32, bw: u32, bh: u32, pos: WinPosition)
    -> WmCtlResult<(Option<u32>, Option<u32>)>
{
    wmctl.unmaximize_win(win)?;

    // Pre-calculations
    let cx = wmctl.work_width/2 - (w + bw)/2;  // center x
    let cy = wmctl.work_height/2 - (h + bh)/2; // center y
    let rx = wmctl.work_width - w - bw;        // right x
    let by = wmctl.work_height - h - bh;       // bottom y

    // Interpret the position as x, y cordinates
    Ok(match pos {
        WinPosition::Center => (Some(cx), Some(cy)),
        WinPosition::Left => (Some(0), None),
        WinPosition::Right => (Some(rx), None),
        WinPosition::Top => (None, Some(0)),
        WinPosition::Bottom => (None, Some(by)),
        WinPosition::TopLeft => (Some(0), Some(0)),
        WinPosition::TopRight => (Some(rx), Some(0)),
        WinPosition::BottomLeft => (Some(0), Some(by)),
        WinPosition::BottomRight => (Some(rx), Some(by)),
        WinPosition::LeftCenter => (Some(0), Some(cy)),
        WinPosition::RightCenter => (Some(rx), Some(cy)),
        WinPosition::TopCenter => (Some(cx), Some(0)),
        WinPosition::BottomCenter => (Some(cx), Some(by)),
    })
}

/// Shape the given window or active window if not given without moving it
fn shape_win(wmctl: &WmCtl, win: u32, w: u32, h: u32, bw: u32, bh: u32, shape: WinShape)
    -> WmCtlResult<(Option<u32>, Option<u32>, Option<u32>)>
{
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
            let w10 = (w as f32*0.1) as u32; // 10% of width
            let h10 = (h as f32*0.1) as u32; // 10% of height

            let (w, h) = match shape {

                // Grow the existing dimensions by 10%
                WinShape::Grow => (Some(w + w10), Some(h + h10)),

                // Resize to a quarter of the work screen
                WinShape::Small => {
                    let w = wmctl.work_width / 2 - bw;
                    let h = wmctl.work_height / 2 - bh;
                    (Some(w), Some(h))
                },

                // Resize to a medium 4x3 window
                WinShape::Medium => {
                    let w = wmctl.width as f32 * 0.55;
                    let h = wmctl.height as f32 * 0.70;
                    shape4x3(w as u32, h as u32, bw, bh)?
                },

                // Resize to a large 4x3 window
                WinShape::Large => {
                    let w = wmctl.width as f32 * 0.75;
                    let h = wmctl.height as f32 * 0.90;
                    shape4x3(w as u32, h as u32, bw, bh)?
                },

                // Shrink the existing dimensions by 10%
                WinShape::Shrink => (Some(w - w10), Some(h - h10)),

                // Resize changing the shorter side to be a 4x3 ratio
                WinShape::Ratio4x3 => shape4x3(w, h, bw, bh)?,

                // Don't change anything by default
                _ => (None, None),
            };
            (Some(WinGravity::Center.into()), w, h)
        }
    })
}

// Resize changing the shorter side to be a 4x3 ratio using, `w` width, `h` height,
// `bw` combined left and right borders, `bh` combined top and bottom borders
fn shape4x3(w: u32, h: u32, bw: u32, bh: u32) -> WmCtlResult<(Option<u32>, Option<u32>)>
{
    let tw = w + bw; // total width
    let th = h + bh; // total height

    let (w, h) = if tw > th {
        (Some(w), Some(((tw - bh) as f32 * 3.0/4.0) as u32))
    } else if th > tw {
        // Offsetting a bit more for borders
        (Some(((th + bh) as f32 * 4.0/3.0) as u32), Some(h))
    } else {
        (None, None)
    };
    Ok((w, h))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
