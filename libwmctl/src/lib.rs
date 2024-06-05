//! `libwmctl` implements a subset of the [Extended Window Manager Hints (EWMH)
//! specification](https://specifications.freedesktop.org/wm-spec/latest/) as a way to integrate
//! with EWMH compatible window managers. The EWHM spec builds on the lower level Inter Client
//! Communication Conventions Manual (ICCCM) to define interactions between window managers,
//! compositing managers and applications.
//!
//! [Root Window Properties](https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html)  
//! The EWMH spec defines a number of properties that EWHM compliant window managers will maintain
//! and return to clients requesting information. `libwmctl` taps into the message queue to retrieve
//! details about a given window and to than manipulate the given window as desired.
//!
//! `wmctl` uses `libwmctl` with pre-defined shapes and positions to manipulate how a window should
//! be shaped and positioned on the screen in an ergonomic way; however `libwmctl` could be used
//! for a variety of use cases separate from wmctl.

mod atoms;
mod error;
mod model;
mod window;
mod winmgr;
pub use atoms::*;
pub use error::*;
pub use model::*;
pub use window::Window;
use winmgr::WinMgr;

/// All essential symbols in a simple consumable form
///
/// ### Examples
/// ```
/// use libwmctl::prelude::*;
/// ```
pub mod prelude {
    pub use crate::*;
}

/// Singleton providing a single instance of WmCtl shared across the application. Using RwLock here
/// since changing the instance won't ever happen and RwLock allows for multiple readers making this
/// as efficient as possible.
use std::sync::{OnceLock, RwLock};
#[allow(non_snake_case)]
fn WM() -> &'static RwLock<WinMgr> {
    static INIT: OnceLock<RwLock<WinMgr>> = OnceLock::new();
    INIT.get_or_init(|| RwLock::new(WinMgr::connect().unwrap()))
}

/// Get window manager informational properties
///
/// ### Examples
/// ```ignore
/// use libwmctl::prelude::*;
/// libwmctl::winmgr().unwrap();
/// ```
pub fn info() -> WmCtlResult<Info> {
    Ok(WM().read().unwrap().info()?)
}

/// Get the active window
///
/// ### Examples
/// ```ignore
/// use libwmctl::prelude::*;
/// let win = libwmctl::active();
/// ```
pub fn active() -> Window {
    Window::from(None)
}

/// Get the window by id
///
/// ### Arguments
/// * `id` - id of the window or None
///
/// ### Examples
/// ```ignore
/// use libwmctl::prelude::*;
/// let win = libwmctl::window(1234);
/// ```
pub fn window(id: u32) -> Window {
    Window::from(Some(id))
}

/// Get all the windows the window manager is managing and their essential properties
///
/// ### Arguments
/// * `hidden` - when set to true will list all x11 windows not just those the window manager lists
///
/// ### Examples
/// ```ignore
/// use libwmctl::prelude::*;
/// libwmctl::windows().unwrap();
/// ```
pub fn windows(hidden: bool) -> WmCtlResult<Vec<Window>> {
    WM().read()
        .unwrap()
        .windows(hidden)?
        .iter()
        .map(|&id| Ok(Window::new(id)))
        .collect::<WmCtlResult<Vec<Window>>>()
}

/// Get the first window that matches the given class
///
/// ### Arguments
/// * `class` - the class to match against
///
/// ### Examples
/// ```ignore
/// use libwmctl::prelude::*;
/// let win = libwmctl::first_by_class("firefox").unwrap();
/// ```
pub fn first_by_class(class: &str) -> Option<Window> {
    let windows = windows(false);
    if windows.is_err() {
        return None;
    }
    windows
        .unwrap()
        .iter()
        .find(|x| x.class().unwrap_or("".to_string()).to_lowercase() == class.to_lowercase())
        .map_or(None, |x| Some(x.clone()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
