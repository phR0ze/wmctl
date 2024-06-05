use std::fmt;

/// WinGravity
/// When windows are resized, subwindows may be repositioned automatically relative to some position
/// in the window. This attraction of a subwindow to some part of its parent is known as window
/// gravity.
///
/// Gravity is defined as the lower byte of the move resize flags 32bit value
/// <https://tronche.com/gui/x/xlib/window/attributes/gravity.html>
#[derive(Debug, Clone, PartialEq)]
pub enum Gravity {
    Unmap,
    Center,
}

// Implement format! support
impl fmt::Display for Gravity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

impl From<u32> for Gravity {
    fn from(val: u32) -> Self {
        match val {
            5 => Gravity::Center,
            _ => Gravity::Unmap,
        }
    }
}

impl From<Gravity> for u32 {
    fn from(val: Gravity) -> Self {
        match val {
            Gravity::Center => 5,
            Gravity::Unmap => 0,
        }
    }
}
