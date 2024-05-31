use std::{collections::HashMap, convert, fmt};
use x11rb::protocol::xproto;

use crate::{atoms::AtomCollection, WmCtlError, WmCtlResult};

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

/// WinMgr provides information about the window manager and its environment.
pub struct Info {
    pub id: u32,
    pub name: String,
    pub compositing: bool,
    pub root_win_id: u32,
    pub work_area: (u32, u32),
    pub screen_size: (u32, u32),
    pub desktops: u32,
    pub supported: HashMap<u32, String>,
}

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

/// WinPosition provides a number of pre-defined positions on the screen to quickly and easily
/// move the window to taking into account borders and taskbars automatically.
#[derive(Debug, Clone, PartialEq)]
pub enum Position {
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
    Static(u32, u32),
}

// Implement format! support
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

// Convert from &str to Postiion
impl convert::TryFrom<&str> for Position {
    type Error = WmCtlError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_ref() {
            "center" => Ok(Position::Center),
            "left" => Ok(Position::Left),
            "right" => Ok(Position::Right),
            "top" => Ok(Position::Top),
            "bottom" => Ok(Position::Bottom),
            "top-left" => Ok(Position::TopLeft),
            "top-right" => Ok(Position::TopRight),
            "bottom-left" => Ok(Position::BottomLeft),
            "bottom-right" => Ok(Position::BottomRight),
            "left-center" => Ok(Position::LeftCenter),
            "right-center" => Ok(Position::RightCenter),
            "top-center" => Ok(Position::TopCenter),
            "bottom-center" => Ok(Position::BottomCenter),
            _ => Err(WmCtlError::InvalidWinPosition(val.to_string()).into()),
        }
    }
}

// Convert from String to Postiion
impl convert::TryFrom<String> for Position {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Position::try_from(val.as_str())
    }
}

/// WinShape provides a number of pre-defined shapes to manipulate the window into, taking into
/// account borders and taskbars automatically.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Grow,
    Max,
    Halfw,
    Halfh,
    Small,
    Medium,
    Large,
    Shrink,
    Square,
    UnMax,
    Static(u32, u32),
}

// Implement format! support
impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

// Convert from &str to Shape
impl convert::TryFrom<&str> for Shape {
    type Error = WmCtlError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_ref() {
            "grow" => Ok(Shape::Grow),
            "max" => Ok(Shape::Max),
            "halfw" => Ok(Shape::Halfw),
            "halfh" => Ok(Shape::Halfh),
            "small" => Ok(Shape::Small),
            "medium" => Ok(Shape::Medium),
            "large" => Ok(Shape::Large),
            "shrink" => Ok(Shape::Shrink),
            "unmax" => Ok(Shape::UnMax),
            _ => Err(WmCtlError::InvalidWinShape(val.to_string()).into()),
        }
    }
}

// Convert from a String to a Shape
impl convert::TryFrom<String> for Shape {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Shape::try_from(val.as_str())
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
impl WinClass {
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
impl WinMap {
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
