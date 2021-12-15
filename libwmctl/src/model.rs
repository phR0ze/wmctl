use crate::{WmCtlResult, WmCtlError, AtomCollection};
use std::{fmt, convert};

use x11rb::protocol::xproto;

/// WinGravity
/// ------------------------------------------------------------------------------------------------

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

/// WinShape
/// ------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum WinShape {
    Grow,
    Shrink,
    Square,
    Ratio4x3,
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
            "shrink" => Ok(WinShape::Shrink),
            "square" => Ok(WinShape::Square),
            "4x3" => Ok(WinShape::Ratio4x3),
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

pub(crate) fn atom_to_string(atoms: &AtomCollection, val: u32) -> WmCtlResult<String> { 
    let property = if val == atoms._NET_ACTIVE_WINDOW {
        "_NET_ACTIVE_WINDOW"
    } else if val == atoms._NET_CLIENT_LIST {
        "_NET_CLIENT_LIST"
    } else if val == atoms._NET_CLIENT_LIST_STACKING {
        "_NET_CLIENT_LIST_STACKING"
    } else if val == atoms._NET_CURRENT_DESKTOP {
         "_NET_CURRENT_DESKTOP"
    } else if val == atoms._NET_CLOSE_WINDOW {
         "_NET_CLOSE_WINDOW"
    } else if val == atoms._NET_DESKTOP_GEOMETRY {
         "_NET_DESKTOP_GEOMETRY"
    } else if val == atoms._NET_DESKTOP_LAYOUT {
         "_NET_DESKTOP_LAYOUT"
    } else if val == atoms._NET_DESKTOP_NAMES {
         "_NET_DESKTOP_NAMES"
    } else if val == atoms._NET_DESKTOP_VIEWPORT {
         "_NET_DESKTOP_VIEWPORT"
    } else if val == atoms._NET_FRAME_EXTENTS {
        "_NET_FRAME_EXTENTS"
    } else if val == atoms._NET_MOVERESIZE_WINDOW {
         "_NET_MOVERESIZE_WINDOW"
    } else if val == atoms._NET_NUMBER_OF_DESKTOPS {
        "_NET_NUMBER_OF_DESKTOPS"
    } else if val == atoms._NET_REQUEST_FRAME_EXTENTS {
         "_NET_REQUEST_FRAME_EXTENTS"
    } else if val == atoms._NET_SHOWING_DESKTOP {
         "_NET_SHOWING_DESKTOP"
    } else if val == atoms._NET_SUPPORTED {
        "_NET_SUPPORTED"
    } else if val == atoms._NET_SUPPORTING_WM_CHECK {
        "_NET_SUPPORTING_WM_CHECK"
    } else if val == atoms._NET_SYSTEM_TRAY_OPCODE {
         "_NET_SYSTEM_TRAY_OPCODE"
    } else if val == atoms._NET_WM_ACTION_ABOVE {
         "_NET_WM_ACTION_ABOVE"
    } else if val == atoms._NET_WM_ACTION_BELOW {
         "_NET_WM_ACTION_BELOW"
    } else if val == atoms._NET_WM_ACTION_CHANGE_DESKTOP {
         "_NET_WM_ACTION_CHANGE_DESKTOP"
    } else if val == atoms._NET_WM_ACTION_CLOSE {
         "_NET_WM_ACTION_CLOSE"
    } else if val == atoms._NET_WM_ACTION_FULLSCREEN {
         "_NET_WM_ACTION_FULLSCREEN"
    } else if val == atoms._NET_WM_ACTION_MAXIMIZE_HORZ {
         "_NET_WM_ACTION_MAXIMIZE_HORZ"
    } else if val == atoms._NET_WM_ACTION_MAXIMIZE_VERT {
         "_NET_WM_ACTION_MAXIMIZE_VERT"
    } else if val == atoms._NET_WM_ACTION_MINIMIZE {
         "_NET_WM_ACTION_MINIMIZE"
    } else if val == atoms._NET_WM_ACTION_MOVE {
         "_NET_WM_ACTION_MOVE"
    } else if val == atoms._NET_WM_ACTION_RESIZE {
         "_NET_WM_ACTION_RESIZE"
    } else if val == atoms._NET_WM_ACTION_SHADE {
         "_NET_WM_ACTION_SHADE"
    } else if val == atoms._NET_WM_ACTION_STICK {
         "_NET_WM_ACTION_STICK"
    } else if val == atoms._NET_WM_ALLOWED_ACTIONS {
         "_NET_WM_ALLOWED_ACTIONS"
    } else if val == atoms._NET_WM_BYPASS_COMPOSITOR {
         "_NET_WM_BYPASS_COMPOSITOR"
    } else if val == atoms._NET_WM_CONTEXT_HELP {
        "_NET_WM_CONTEXT_HELP"
    } else if val == atoms._NET_WM_DESKTOP {
        "_NET_WM_DESKTOP"
    } else if val == atoms._NET_WM_FULLSCREEN_MONITORS {
         "_NET_WM_FULLSCREEN_MONITORS"
    } else if val == atoms._NET_WM_HANDLED_ICONS {
         "_NET_WM_HANDLED_ICONS"
    } else if val == atoms._NET_WM_ICON {
         "_NET_WM_ICON"
    } else if val == atoms._NET_WM_ICON_GEOMETRY {
         "_NET_WM_ICON_GEOMETRY"
    } else if val == atoms._NET_WM_ICON_NAME {
         "_NET_WM_ICON_NAME"
    } else if val == atoms._NET_WM_NAME {
        "_NET_WM_NAME"
    } else if val == atoms._NET_WM_OPAQUE_REGION {
         "_NET_WM_OPAQUE_REGION"
    } else if val == atoms._NET_WM_PID {
        "_NET_WM_PID"
    } else if val == atoms._NET_WM_PING {
         "_NET_WM_PING"
    } else if val == atoms._NET_WM_WINDOW_OPACITY {
        "_NET_WM_WINDOW_OPACITY"
    } else if val == atoms._NET_WM_WINDOW_OPACITY_LOCKED {
        "_NET_WM_WINDOW_OPACITY_LOCKED"
    } else if val == atoms._NET_WM_STATE {
        "_NET_WM_STATE"
    } else if val == atoms._NET_WM_STATE_ABOVE {
        "_NET_WM_STATE_ABOVE"
    } else if val == atoms._NET_WM_STATE_BELOW {
        "_NET_WM_STATE_BELOW"
    } else if val == atoms._NET_WM_STATE_DEMANDS_ATTENTION {
        "_NET_WM_STATE_DEMANDS_ATTENTION"
    } else if val == atoms._NET_WM_STATE_FOCUSED {
        "_NET_WM_STATE_FOCUSED"
    } else if val == atoms._NET_WM_STATE_FULLSCREEN {
        "_NET_WM_STATE_FULLSCREEN"
    } else if val == atoms._NET_WM_MOVERESIZE {
         "_NET_WM_MOVERESIZE"
    } else if val == atoms._NET_WM_STATE_HIDDEN {
        "_NET_WM_STATE_HIDDEN"
    } else if val == atoms._NET_WM_STATE_MAXIMIZED_VERT {
        "_NET_WM_STATE_MAXIMIZED_VERT"
    } else if val == atoms._NET_WM_STATE_MAXIMIZED_HORZ {
        "_NET_WM_STATE_MAXIMIZED_HORZ"
    } else if val == atoms._NET_WM_STATE_MODAL {
        "_NET_WM_STATE_MODAL"
    } else if val == atoms._NET_WM_STATE_SHADED {
        "_NET_WM_STATE_SHADED"
    } else if val == atoms._NET_WM_STATE_SKIP_PAGER {
        "_NET_WM_STATE_SKIP_PAGER"
    } else if val == atoms._NET_WM_STATE_SKIP_TASKBAR {
        "_NET_WM_STATE_SKIP_TASKBAR"
    } else if val == atoms._NET_WM_STATE_STICKY {
        "_NET_WM_STATE_STICKY"
    } else if val == atoms._NET_WM_STRUT {
        "_NET_WM_STRUT"
    } else if val == atoms._NET_WM_STRUT_PARTIAL {
        "_NET_WM_STRUT_PARTIAL"
    } else if val == atoms._NET_WM_SYNC_REQUEST {
        "_NET_WM_SYNC_REQUEST"
    } else if val == atoms._NET_WM_SYNC_REQUEST_COUNTER {
        "_NET_WM_SYNC_REQUEST_COUNTER"
    } else if val == atoms._NET_WM_USER_TIME {
         "_NET_WM_USER_TIME"
    } else if val == atoms._NET_WM_USER_TIME_WINDOW {
         "_NET_WM_USER_TIME_WINDOW"
    } else if val == atoms._NET_WM_VISIBLE_ICON_NAME {
        "_NET_WM_VISIBLE_ICON_NAME"
    } else if val == atoms._NET_WM_VISIBLE_NAME {
        "_NET_WM_VISIBLE_NAME"
    } else if val == atoms._NET_WM_WINDOW_TYPE {
        "_NET_WM_WINDOW_TYPE"
    } else if val == atoms._NET_WM_WINDOW_TYPE_COMBO {
        "_NET_WM_WINDOW_TYPE_COMBO"
    } else if val == atoms._NET_WM_WINDOW_TYPE_DESKTOP {
        "_NET_WM_WINDOW_TYPE_DESKTOP"
    } else if val == atoms._NET_WM_WINDOW_TYPE_DIALOG {
        "_NET_WM_WINDOW_TYPE_DIALOG"
    } else if val == atoms._NET_WM_WINDOW_TYPE_DND {
        "_NET_WM_WINDOW_TYPE_DND"
    } else if val == atoms._NET_WM_WINDOW_TYPE_DOCK {
        "_NET_WM_WINDOW_TYPE_DOCK"
    } else if val == atoms._NET_WM_WINDOW_TYPE_DROPDOWN_MENU {
        "_NET_WM_WINDOW_TYPE_DROPDOWN_MENU"
    } else if val == atoms._NET_WM_WINDOW_TYPE_MENU {
        "_NET_WM_WINDOW_TYPE_MENU"
    } else if val == atoms._NET_WM_WINDOW_TYPE_NORMAL {
        "_NET_WM_WINDOW_TYPE_NORMAL"
    } else if val == atoms._NET_WM_WINDOW_TYPE_NOTIFICATION {
        "_NET_WM_WINDOW_TYPE_NOTIFICATION"
    } else if val == atoms._NET_WM_WINDOW_TYPE_POPUP_MENU {
        "_NET_WM_WINDOW_TYPE_POPUP_MENU"
    } else if val == atoms._NET_WM_WINDOW_TYPE_SPLASH {
        "_NET_WM_WINDOW_TYPE_SPLASH"
    } else if val == atoms._NET_WM_WINDOW_TYPE_TOOLBAR {
        "_NET_WM_WINDOW_TYPE_TOOLBAR"
    } else if val == atoms._NET_WM_WINDOW_TYPE_TOOLTIP {
        "_NET_WM_WINDOW_TYPE_TOOLTIP"
    } else if val == atoms._NET_WM_WINDOW_TYPE_UTILITY {
        "_NET_WM_WINDOW_TYPE_UTILITY"
    } else if val == atoms._NET_WORKAREA {
        "_NET_WORKAREA"
    } else if val == atoms.UTF8_STRING {
        "UTF8_STRING"
    } else {
        Err(WmCtlError::InvalidAtom(val.to_string()))?
    };
    Ok(property.to_owned())
}