//! Models for working with X11 windows
//!
//! ### How to use the `model` module
//! ```
//! use libwmctl::prelude::*;
//! ```
mod class;
mod gravity;
mod info;
mod kind;
mod map_state;
mod position;
mod property;
mod shape;
mod state;

// Export contents of modules
pub use class::*;
pub use gravity::*;
pub use info::*;
pub use kind::*;
pub use map_state::*;
pub use position::*;
pub use property::*;
pub use shape::*;
pub use state::*;

// Define the second byte of the move resize flags 32bit value
// Used to indicate that the associated value has been changed and needs to be acted upon
pub type MoveResizeWindowFlags = u32;
pub const MOVE_RESIZE_WINDOW_X: MoveResizeWindowFlags = 1 << 8;
pub const MOVE_RESIZE_WINDOW_Y: MoveResizeWindowFlags = 1 << 9;
pub const MOVE_RESIZE_WINDOW_WIDTH: MoveResizeWindowFlags = 1 << 10;
pub const MOVE_RESIZE_WINDOW_HEIGHT: MoveResizeWindowFlags = 1 << 11;

pub type WindowStateAction = u32;
pub const WINDOW_STATE_ACTION_REMOVE: WindowStateAction = 0;
pub const WINDOW_STATE_ACTION_ADD: WindowStateAction = 1;

/// Rect provides a simple way to store the width and height of an area
pub struct Rect {
    pub w: u32,
    pub h: u32,
}

/// Coord provides a simple way to store x, y coordinates
pub struct Coord {
    pub x: u32,
    pub y: u32,
}

/// CoordOpt provides a simple way to store optional x, y coordinates
pub struct CoordOpt {
    pub x: Option<u32>,
    pub y: Option<u32>,
}
