use crate::{atoms::AtomCollection, WmCtlError, WmCtlResult};
use std::fmt;

/// State provides an easy way to identify the different window states
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Above,            // show the window above others
    Below,            // show the window below others
    DemandsAttention, // same as the urgent flag
    Focused,          // the window has input focus
    Fullscreen,       // show the window fullscreen
    Hidden,           // the window is unmapped
    MaxHorz,          // the window is maximized horizontally
    MaxVert,          // the window is maximized vertically
    Modal,            // the window is a modal dialog
    Shaded,           // the window is rolled up
    SkipPager,        // the window should not be shown on a pager
    SkipTaskbar,      // the window should be ignored by a taskbar
    Sticky,           // the window should be shown on all virtual desktops
    Invalid,          // made up value to track missing
}

// Convert from u32 to State
impl State {
    pub fn from(atoms: &AtomCollection, val: u32) -> WmCtlResult<State> {
        if val == atoms._NET_WM_STATE_ABOVE {
            Ok(State::Above)
        } else if val == atoms._NET_WM_STATE_BELOW {
            Ok(State::Below)
        } else if val == atoms._NET_WM_STATE_DEMANDS_ATTENTION {
            Ok(State::DemandsAttention)
        } else if val == atoms._NET_WM_STATE_FOCUSED {
            Ok(State::Focused)
        } else if val == atoms._NET_WM_STATE_FULLSCREEN {
            Ok(State::Fullscreen)
        } else if val == atoms._NET_WM_STATE_HIDDEN {
            Ok(State::Hidden)
        } else if val == atoms._NET_WM_STATE_MAXIMIZED_HORZ {
            Ok(State::MaxHorz)
        } else if val == atoms._NET_WM_STATE_MAXIMIZED_VERT {
            Ok(State::MaxVert)
        } else if val == atoms._NET_WM_STATE_MODAL {
            Ok(State::Modal)
        } else if val == atoms._NET_WM_STATE_SHADED {
            Ok(State::Shaded)
        } else if val == atoms._NET_WM_STATE_SKIP_PAGER {
            Ok(State::SkipPager)
        } else if val == atoms._NET_WM_STATE_SKIP_TASKBAR {
            Ok(State::SkipTaskbar)
        } else if val == atoms._NET_WM_STATE_STICKY {
            Ok(State::Sticky)
        } else {
            Err(WmCtlError::InvalidWinState(val).into())
        }
    }
}

// Implement format! support
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            State::Invalid => write!(f, ""),
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}
