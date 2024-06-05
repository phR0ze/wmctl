use crate::{atoms::AtomCollection, WmCtlError, WmCtlResult};
use std::fmt;

/// WinState provides an easy way to identify the different window states
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Above,
    Below,
    DemandsAttention,
    Focused,
    Fullscreen,
    Hidden,
    MaxVert,
    MaxHorz,
    Modal,
    Shaded,
    SkipPager,
    SkipTaskbar,
    Invalid, // made up value to track missing
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
        } else if val == atoms._NET_WM_STATE_MAXIMIZED_VERT {
            Ok(State::MaxVert)
        } else if val == atoms._NET_WM_STATE_MAXIMIZED_HORZ {
            Ok(State::MaxHorz)
        } else if val == atoms._NET_WM_STATE_MODAL {
            Ok(State::Modal)
        } else if val == atoms._NET_WM_STATE_SHADED {
            Ok(State::Shaded)
        } else if val == atoms._NET_WM_STATE_SKIP_PAGER {
            Ok(State::SkipPager)
        } else if val == atoms._NET_WM_STATE_SKIP_TASKBAR {
            Ok(State::SkipTaskbar)
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
