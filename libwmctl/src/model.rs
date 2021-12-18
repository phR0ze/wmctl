use crate::{WmCtlResult, WmCtlError, wmctl::AtomCollection};
use std::{fmt, convert};

use x11rb::protocol::xproto;

/// WinGravity
/// Gravity is defined as the lower byte of the move resize flags 32bit value
/// https://tronche.com/gui/x/xlib/window/attributes/gravity.html
#[derive(Debug, Clone, PartialEq)]
pub enum WinGravity {
    Center,
    None,
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
            _ => WinGravity::None,
        }
    }
}

impl From<WinGravity> for u32 {
    fn from(val: WinGravity) -> Self {
        match val {
            WinGravity::Center => 5,
            _ => 0,
        }
    }
}

/// WinPosition provides a number of pre-defined positions on the screen to quickly and easily
/// move the window to taking into account borders and taskbars automatically.
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
    LeftCenter,
    RightCenter,
    TopCenter,
    BottomCenter,
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
impl convert::TryFrom<&str> for WinPosition {
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
            "left-center" => Ok(WinPosition::LeftCenter),
            "right-center" => Ok(WinPosition::RightCenter),
            "top-center" => Ok(WinPosition::TopCenter),
            "bottom-center" => Ok(WinPosition::BottomCenter),
            _ => Err(WmCtlError::InvalidWinPosition(val.to_string()).into()),
        }
    }
}

// Convert from String to Postiion
impl convert::TryFrom<String> for WinPosition {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        WinPosition::try_from(val.as_str())
    }
}

/// WinShape provides a number of pre-defined shapes to manipulate the window into, taking into
/// account borders and taskbars automatically.
#[derive(Debug, Clone, PartialEq)]
pub enum WinShape {
    Grow,
    Max,
    Halfw,
    Halfh,
    Small,
    Medium,
    Large,
    Shrink,
    Square,
    Ratio4x3,
    UnMax,
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
impl convert::TryFrom<&str> for WinShape {
    type Error = WmCtlError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_ref() {
            "grow" => Ok(WinShape::Grow),
            "max" => Ok(WinShape::Max),
            "halfw" => Ok(WinShape::Halfw),
            "halfh" => Ok(WinShape::Halfh),
            "small" => Ok(WinShape::Small),
            "medium" => Ok(WinShape::Medium),
            "large" => Ok(WinShape::Large),
            "shrink" => Ok(WinShape::Shrink),
            "4x3" => Ok(WinShape::Ratio4x3),
            "unmax" => Ok(WinShape::UnMax),
            _ => Err(WmCtlError::InvalidWinShape(val.to_string()).into()),
        }
    }
}

// Convert from a String to a Shape
impl convert::TryFrom<String> for WinShape {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        WinShape::try_from(val.as_str())
    }
}

/// WinClass provides a easy way to identify the different window class types
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WinClass {
    CopyFromParent,
    InputOnly,
    InputOutput,
}

// Convert from u32 to Class
impl WinClass
{
    pub fn from(val: u32) -> WmCtlResult<WinClass> {
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

/// WinMap provides an easy way to identify the differnt window map values
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WinMap {
    Unmapped,
    Unviewable,
    Viewable,
}

// Convert from u32 to state
impl WinMap
{
    pub fn from(val: u32) -> WmCtlResult<WinMap> {
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
impl WinState
{
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

/// WinType provides an easy way to identify the different window types
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WinType {
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
    pub fn from(atoms: &AtomCollection, val: u32) -> WmCtlResult<WinType> {
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