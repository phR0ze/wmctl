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

/// Border provides a simple way to store border values
#[derive(Default)]
pub struct Border {
    pub l: u32,
    pub r: u32,
    pub t: u32,
    pub b: u32,
}

impl Border {
    pub fn new(l: u32, r: u32, t: u32, b: u32) -> Self {
        Self { l, r, t, b }
    }

    // Check if any values are non zero
    pub fn any(&self) -> bool {
        self.l > 0 || self.r > 0 || self.t > 0 || self.b > 0
    }

    // Summed the left and right borders as a single value
    pub fn w(&self) -> u32 {
        self.l + self.r
    }

    // Summed the top and bottom borders as a single value
    pub fn h(&self) -> u32 {
        self.t + self.b
    }
}

/// Rect provides a simple way to store the width and height of an area
#[derive(Default)]
pub struct Rect {
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn new(w: u32, h: u32) -> Self {
        Self { w, h }
    }
}
