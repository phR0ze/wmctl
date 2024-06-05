use crate::{atoms::AtomCollection, WmCtlError, WmCtlResult};
use std::fmt;

/// WinState provides an easy way to identify the different window states
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WinState {
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
impl WinState {
    pub fn from(atoms: &AtomCollection, val: u32) -> WmCtlResult<WinState> {
        if val == atoms._NET_WM_STATE_ABOVE {
            Ok(WinState::Above)
        } else if val == atoms._NET_WM_STATE_BELOW {
            Ok(WinState::Below)
        } else if val == atoms._NET_WM_STATE_DEMANDS_ATTENTION {
            Ok(WinState::DemandsAttention)
        } else if val == atoms._NET_WM_STATE_FOCUSED {
            Ok(WinState::Focused)
        } else if val == atoms._NET_WM_STATE_FULLSCREEN {
            Ok(WinState::Fullscreen)
        } else if val == atoms._NET_WM_STATE_HIDDEN {
            Ok(WinState::Hidden)
        } else if val == atoms._NET_WM_STATE_MAXIMIZED_VERT {
            Ok(WinState::MaxVert)
        } else if val == atoms._NET_WM_STATE_MAXIMIZED_HORZ {
            Ok(WinState::MaxHorz)
        } else if val == atoms._NET_WM_STATE_MODAL {
            Ok(WinState::Modal)
        } else if val == atoms._NET_WM_STATE_SHADED {
            Ok(WinState::Shaded)
        } else if val == atoms._NET_WM_STATE_SKIP_PAGER {
            Ok(WinState::SkipPager)
        } else if val == atoms._NET_WM_STATE_SKIP_TASKBAR {
            Ok(WinState::SkipTaskbar)
        } else {
            Err(WmCtlError::InvalidWinState(val).into())
        }
    }
}

// Implement format! support
impl fmt::Display for WinState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WinState::Invalid => write!(f, ""),
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}
