mod wmctl;
mod error;
mod model;
use wmctl::*;
use error::*;
use model::*;

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
pub fn info() -> WmCtlResult<()> {
    let wmctl = WmCtl::connect()?;
    let (_, wm_name) = wmctl.winmgr()?;
    let win = wmctl.active_win()?;
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
pub fn list(all: bool) -> WmCtlResult<()> {
    let wmctl = WmCtl::connect()?;
    print_win_header();
    for win in wmctl.windows(all)? {
        print_win_details(&wmctl, win)?;
    }
    Ok(())
}

fn print_win_header() {
    println!("{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}", "ID", "DSK", "PID", "X", "Y", "W", "H", "BORDERS", "TYPE", "STATE", "CLASS", "NAME");
}

fn print_win_details(wmctl: &WmCtl, win: u32) -> WmCtlResult<()> {
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

/// Move the active window without changing its size
pub fn move_win(pos: WinPosition) -> WmCtlResult<()> {
    let wmctl = WmCtl::connect()?;

    // Get the current window
    let win = wmctl.active_win()?;
    let (_, _, w, h) = wmctl.win_geometry(win)?;
    let (bl, br, bt, bb) = wmctl.win_borders(win)?;

    // Pre-calculations
    let cx = wmctl.work_width/2 - (w + bl + br)/2;  // center x
    let cy = wmctl.work_height/2 - (h + bt + bb)/2; // center y
    let rx = wmctl.work_width - w - bl - br;        // right x
    let by = wmctl.work_height - h - bt - bb;       // bottom y

    // Interpret the position as x, y cordinates
    let (x, y) = match pos {
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
    };

    // Move the current window as indicated
    wmctl.move_resize_win(win, None, x, y, None, None)?;
    Ok(())
}

/// Resize the active window based on the ratio of the overall screen size then center it
pub fn resize_and_center(x_ratio: u32, y_ratio: u32) -> WmCtlResult<()> {
    // let x_ratio = x_ratio as f64 * 0.01;
    // let y_ratio = y_ratio as f64 * 0.01;
    // let wmctl = WmCtl::connect()?;
    // let win = wmctl.active_win()?;

    // // Remove maximizing states
    // wmctl.win_remove_maximize(win)?;

    // // Calculate window size
    // let (w, h) =  ((wmctl.work_width as f64 * x_ratio) as i32, (wmctl.work_height as f64 * y_ratio) as i32);

    // // Center the window on the screen
    // let (x, y) =  ((wmctl.work_width - w)/2, (wmctl.work_height - h)/2);

    // wmctl.move_and_resize(win, x, y, w, h)?;
    Ok(())
}

/// Shape the active window without moving it
pub fn shape_win(shape: WinShape) -> WmCtlResult<()> {
    let wmctl = WmCtl::connect()?;

    // Get the current window
    let win = wmctl.active_win()?;
    let (_, _, w, h) = wmctl.win_geometry(win)?;
    let (bl, br, bt, bb) = wmctl.win_borders(win)?;
    // wmctl.win_remove_maximize(win)?;

    // Pre-calculations
    let tw = w + bl + br; // total width
    let th = h + bt + bb; // total height
    let w10 = (w as f32*0.1) as u32; // 10% of width
    let h10 = (h as f32*0.1) as u32; // 10% of height

    let (w, h) = match shape {
        WinShape::Grow => (Some(w + w10), Some(h + h10)),
        WinShape::Shrink => (Some(w - w10), Some(h - h10)),
        WinShape::Square => {
            if tw > th {
                (None, Some(tw - bt - bb))
            } else if th > tw {
                (Some(th + bt + bb), None)
            } else {
                (None, None)
            }
        },
        WinShape::Ratio4x3 => {
            if tw > th {
                (None, Some(((tw - bt - bb) as f32 * 3.0/4.0) as u32))
            } else if th > tw {
                (Some(((th + bt + bb) as f32 * 4.0/3.0) as u32), None)
            } else {
                (None, None)
            }
        },
        _ => (None, None),
    };

    wmctl.move_resize_win(win, Some(WinGravity::Center.into()), None, None, w, h)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
