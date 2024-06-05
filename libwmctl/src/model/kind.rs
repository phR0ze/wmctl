use crate::{atoms::AtomCollection, WmCtlError, WmCtlResult};
use std::fmt;

/// WinType provides an easy way to identify the different window types
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WinKind {
    Combo,
    Desktop,
    Dialog,
    DND,
    Dock,
    DropDownMenu,
    Menu,
    Normal,
    Notification,
    PopupMenu,
    Splash,
    Toolbar,
    ToolTip,
    Utility,
    Invalid, // made up value to track missing
}

// Convert from u32 to Type
impl WinKind {
    pub fn from(atoms: &AtomCollection, val: u32) -> WmCtlResult<WinKind> {
        if val == atoms._NET_WM_WINDOW_TYPE_COMBO {
            Ok(WinKind::Combo)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DESKTOP {
            Ok(WinKind::Desktop)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DIALOG {
            Ok(WinKind::Dialog)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DND {
            Ok(WinKind::DND)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DOCK {
            Ok(WinKind::Dock)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DROPDOWN_MENU {
            Ok(WinKind::DropDownMenu)
        } else if val == atoms._NET_WM_WINDOW_TYPE_MENU {
            Ok(WinKind::Menu)
        } else if val == atoms._NET_WM_WINDOW_TYPE_NORMAL {
            Ok(WinKind::Normal)
        } else if val == atoms._NET_WM_WINDOW_TYPE_NOTIFICATION {
            Ok(WinKind::Notification)
        } else if val == atoms._NET_WM_WINDOW_TYPE_POPUP_MENU {
            Ok(WinKind::PopupMenu)
        } else if val == atoms._NET_WM_WINDOW_TYPE_SPLASH {
            Ok(WinKind::Splash)
        } else if val == atoms._NET_WM_WINDOW_TYPE_TOOLBAR {
            Ok(WinKind::Toolbar)
        } else if val == atoms._NET_WM_WINDOW_TYPE_TOOLTIP {
            Ok(WinKind::ToolTip)
        } else if val == atoms._NET_WM_WINDOW_TYPE_UTILITY {
            Ok(WinKind::Utility)
        } else {
            Err(WmCtlError::InvalidWinType(val).into())
        }
    }
}

// Implement format! support
impl fmt::Display for WinKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WinKind::Invalid => write!(f, ""),
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}
