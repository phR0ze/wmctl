mod display;
mod error;
mod position;
mod shape;
use display::*;
use error::*;
use shape::*;
use position::*;

/// All essential symbols in a simple consumable form
///
/// ### Examples
/// ```
/// use libwmctl::prelude::*;
/// ```
pub mod prelude {
    pub use crate::*;
    pub use error::*;
    pub use shape::*;
    pub use position::*;
}

/// Move the active window without changing its size
pub fn move_win(position: Position) -> WmCtlResult<()> {
    let display = Display::open()?;
    let win = display.active_win()?;
    display.remove_maximize(win)?;

    // Value returned for y is 28 off??
    // let (x, y, w, h) = display.win_geometry(win)?;
    // println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);

    // // right
    // let x = display.work_width - w;
    // let y = 0; // seems to be 28 off by default?
    // //let y = display.work_height - h;
    // println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);

    // println!("w: {}, h: {}", display.full_width, display.full_height);
    // println!("w: {}, h: {}", display.work_width, display.work_height);

    display.move_win(win, 500, 0)?;
    Ok(())
}

/// Resize the active window based on the ratio of the overall screen size then center it
pub fn resize_and_center(x_ratio: u32, y_ratio: u32) -> WmCtlResult<()> {
    let x_ratio = x_ratio as f64 * 0.01;
    let y_ratio = y_ratio as f64 * 0.01;
    let display = Display::open()?;
    let win = display.active_win()?;

    // Remove maximizing states
    display.remove_maximize(win)?;

    // Calculate window size
    let (w, h) =  ((display.work_width as f64 * x_ratio) as i32, (display.work_height as f64 * y_ratio) as i32);

    // Center the window on the screen
    let (x, y) =  ((display.work_width - w)/2, (display.work_height - h)/2);

    display.move_and_resize(win, x, y, w, h)?;
    Ok(())
}

/// Shape the active window without moving it
pub fn shape_win(shape: Shape) -> WmCtlResult<()> {
    let display = Display::open()?;
    let win = display.active_win()?;
    display.remove_maximize(win)?;

    // Set longer side to shorter side
    //let 4x3 = 

    let (x, y, mut w, mut h) = display.win_geometry(win)?;
    println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);
    if h < w {
        w = h;
    }
    if w < h {
        h = w;
    }

    // Value returned for y is 28 off??
    println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);
    println!("w: {}, h: {}", display.full_width, display.full_height);
    println!("w: {}, h: {}", display.work_width, display.work_height);

    display.move_and_resize(win, x, y, w, h)?;
    Ok(())
}

/// List out all the current window ids and their titles
pub fn list_windows() -> WmCtlResult<()> {
    let display = Display::open()?;
    for (id, name) in display.windows()? {
        let (_, _, w, h) = display.win_geometry(id)?;
        println!("ID: {}, Size: {}x{}, Name: {}", id, w, h, name);
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
