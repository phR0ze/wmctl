use crate::WmCtlError;
use std::fmt;

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

// Win Error
// -------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WinError {
    TaskbarNotFound,
    TaskbarReservationNotFound,
}
impl std::error::Error for WinError {}
impl fmt::Display for WinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WinError::TaskbarNotFound => write!(f, "taskbar not found"),
            WinError::TaskbarReservationNotFound => write!(f, "taskbar reservation not found"),
        }
    }
}

impl From<WinError> for WmCtlError {
    fn from(err: WinError) -> WmCtlError {
        WmCtlError::Win(err)
    }
}
