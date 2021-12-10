use crate::{WmCtlResult, WmCtlError, AtomCollection};
use std::{fmt, convert::TryFrom};

use x11rb::protocol::xproto;

/// Win
/// ------------------------------------------------------------------------------------------------
#[allow(dead_code)]
pub(crate) struct Win {
    pub(crate) id: u32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: i32,
    pub(crate) h: i32,
}

impl Default for Win {
    fn default() -> Self {
        Self {
            id: Default::default(),
            x: Default::default(),
            y: Default::default(),
            w: Default::default(),
            h: Default::default(),
        }
    }
}

/// WinPosition
/// ------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum WinPosition {
    Center,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

// Implement format! support
impl fmt::Display for WinPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

// Convert from &str to Postiion
impl TryFrom<&str> for WinPosition {
    type Error = WmCtlError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_ref() {
            "center" => Ok(WinPosition::Center),
            "left" => Ok(WinPosition::Left),
            "right" => Ok(WinPosition::Right),
            "top" => Ok(WinPosition::Top),
            "bottom" => Ok(WinPosition::Bottom),
            "top-left" => Ok(WinPosition::TopLeft),
            "top-right" => Ok(WinPosition::TopRight),
            "bottom-left" => Ok(WinPosition::BottomLeft),
            "bottom-right" => Ok(WinPosition::BottomRight),
            _ => Err(WmCtlError::InvalidPosition(val.to_string()).into()),
        }
    }
}

// Convert from String to Postiion
impl TryFrom<String> for WinPosition {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        WinPosition::try_from(val.as_str())
    }
}

/// WinShape
/// ------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum WinShape {
    Square,
}

// Implement format! support
impl fmt::Display for WinShape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

// Convert from &str to Shape
impl TryFrom<&str> for WinShape {
    type Error = WmCtlError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_ref() {
            "square" => Ok(WinShape::Square),
            _ => Err(WmCtlError::InvalidShape(val.to_string()).into()),
        }
    }
}

// Convert from a String to a Shape
impl TryFrom<String> for WinShape {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        WinShape::try_from(val.as_str())
    }
}

/// WinClass
/// ------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WinClass {
    CopyFromParent,
    InputOnly,
    InputOutput,
}

// Convert from u32 to Class
impl WinClass
{
    pub(crate) fn from(val: u32) -> WmCtlResult<WinClass> {
        if val == xproto::WindowClass::COPY_FROM_PARENT.into() {
            Ok(WinClass::CopyFromParent)
        } else if val == xproto::WindowClass::INPUT_ONLY.into() {
            Ok(WinClass::InputOnly)
        } else if val == xproto::WindowClass::INPUT_OUTPUT.into() {
            Ok(WinClass::InputOutput)
        } else {
            Err(WmCtlError::InvalidWinClass(val).into())
        }
    }
}

// Implement format! support
impl fmt::Display for WinClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

/// WinState
/// ------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WinMap {
    Unmapped,
    Unviewable,
    Viewable,
}

// Convert from u32 to state
impl WinMap
{
    pub(crate) fn from(val: u32) -> WmCtlResult<WinMap> {
        if val == xproto::MapState::UNMAPPED.into() {
            Ok(WinMap::Unmapped)
        } else if val == xproto::MapState::UNVIEWABLE.into() {
            Ok(WinMap::Unviewable)
        } else if val == xproto::MapState::VIEWABLE.into() {
            Ok(WinMap::Viewable)
        } else {
            Err(WmCtlError::InvalidWinMap(val).into())
        }
    }
}

// Implement format! support
impl fmt::Display for WinMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

/// WinState
/// ------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WinState {
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
impl WinState
{
    pub(crate) fn from(atoms: &AtomCollection, val: u32) -> WmCtlResult<WinState> {
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

/// WinType
/// ------------------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WinType {
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
impl WinType
{
    pub(crate) fn from(atoms: &AtomCollection, val: u32) -> WmCtlResult<WinType> {
        if val == atoms._NET_WM_WINDOW_TYPE_COMBO {
            Ok(WinType::Combo)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DESKTOP {
            Ok(WinType::Desktop)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DIALOG {
            Ok(WinType::Dialog)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DND {
            Ok(WinType::DND)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DOCK {
            Ok(WinType::Dock)
        } else if val == atoms._NET_WM_WINDOW_TYPE_DROPDOWN_MENU {
            Ok(WinType::DropDownMenu)
        } else if val == atoms._NET_WM_WINDOW_TYPE_MENU {
            Ok(WinType::Menu)
        } else if val == atoms._NET_WM_WINDOW_TYPE_NORMAL {
            Ok(WinType::Normal)
        } else if val == atoms._NET_WM_WINDOW_TYPE_NOTIFICATION {
            Ok(WinType::Notification)
        } else if val == atoms._NET_WM_WINDOW_TYPE_POPUP_MENU {
            Ok(WinType::PopupMenu)
        } else if val == atoms._NET_WM_WINDOW_TYPE_SPLASH {
            Ok(WinType::Splash)
        } else if val == atoms._NET_WM_WINDOW_TYPE_TOOLBAR {
            Ok(WinType::Toolbar)
        } else if val == atoms._NET_WM_WINDOW_TYPE_TOOLTIP {
            Ok(WinType::ToolTip)
        } else if val == atoms._NET_WM_WINDOW_TYPE_UTILITY {
            Ok(WinType::Utility)
        } else {
            Err(WmCtlError::InvalidWinType(val).into())
        }
    }
}

// Implement format! support
impl fmt::Display for WinType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WinType::Invalid => write!(f, ""),
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}
