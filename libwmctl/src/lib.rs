mod error;
mod position;
mod shape;
mod win;
mod wmctl;

use error::*;
use shape::*;
use position::*;
use win::*;
use wmctl::*;

/// All essential symbols in a simple consumable form
///
/// ### Examples
/// ```
/// use libwmctl::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{error::*, shape::*, position::*};
}

/// List out all the current window ids and their titles
pub fn list_windows() -> WmCtlResult<()> {
    let wmctl = WmCtl::connect()?;
    wmctl.windows()?;
    Ok(())
}

/// Move the active window without changing its size
pub fn move_win(position: Position) -> WmCtlResult<()> {
    let wmctl = WmCtl::connect()?;
    wmctl.windows()?;
    // let win = wmctl.active_win()?;
    // wmctl.move_win(win, position)?;
    Ok(())
}

/// Resize the active window based on the ratio of the overall screen size then center it
pub fn resize_and_center(x_ratio: u32, y_ratio: u32) -> WmCtlResult<()> {
    // let x_ratio = x_ratio as f64 * 0.01;
    // let y_ratio = y_ratio as f64 * 0.01;
    // let wmctl = WmCtl::connect()?;
    // let win = wmctl.active_win()?;

    // // Remove maximizing states
    // wmctl.remove_maximize(win)?;

    // // Calculate window size
    // let (w, h) =  ((wmctl.work_width as f64 * x_ratio) as i32, (wmctl.work_height as f64 * y_ratio) as i32);

    // // Center the window on the screen
    // let (x, y) =  ((wmctl.work_width - w)/2, (wmctl.work_height - h)/2);

    // wmctl.move_and_resize(win, x, y, w, h)?;
    Ok(())
}

/// Shape the active window without moving it
pub fn shape_win(shape: Shape) -> WmCtlResult<()> {
    // let wmctl = WmCtl::connect()?;
    // let win = wmctl.active_win()?;
    // wmctl.remove_maximize(win)?;

    // // Set longer side to shorter side
    // //let 4x3 = 

    // let (x, y, mut w, mut h) = wmctl.win_geometry(win)?;
    // println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);
    // if h < w {
    //     w = h;
    // }
    // if w < h {
    //     h = w;
    // }

    // // Value returned for y is 28 off??
    // println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);
    // println!("w: {}, h: {}", wmctl.full_width, wmctl.full_height);
    // println!("w: {}, h: {}", wmctl.work_width, wmctl.work_height);

    // wmctl.move_and_resize(win, x, y, w, h)?;
    Ok(())
}

/// Get screen resolution
pub fn resolution() -> WmCtlResult<(u32, u32)> {
    // let wmctl = WmCtl::connect()?;
    // Ok((wmctl.full_width as u32, wmctl.full_height as u32))
    Ok((0, 0))
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
