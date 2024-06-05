use std::fmt;

/// WinGravity
/// When windows are resized, subwindows may be repositioned automatically relative to some position
/// in the window. This attraction of a subwindow to some part of its parent is known as window
/// gravity.
///
/// Gravity is defined as the lower byte of the move resize flags 32bit value
/// <https://tronche.com/gui/x/xlib/window/attributes/gravity.html>
#[derive(Debug, Clone, PartialEq)]
pub enum WinGravity {
    Unmap,
    Center,
}

// Implement format! support
impl fmt::Display for WinGravity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

impl From<u32> for WinGravity {
    fn from(val: u32) -> Self {
        match val {
            5 => WinGravity::Center,
            _ => WinGravity::Unmap,
        }
    }
}

impl From<WinGravity> for u32 {
    fn from(val: WinGravity) -> Self {
        match val {
            WinGravity::Center => 5,
            WinGravity::Unmap => 0,
        }
    }
}
