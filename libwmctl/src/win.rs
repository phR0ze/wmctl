use crate::WmCtlError;
use std::fmt;

pub(crate) struct Win {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: i32,
    pub(crate) h: i32,
}

// Win Error
// -------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WinError {
    TaskbarNotFound,
}
impl std::error::Error for WinError {}
impl fmt::Display for WinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WinError::TaskbarNotFound => write!(f, "taskbar not found"),
        }
    }
}

impl From<WinError> for WmCtlError {
    fn from(err: WinError) -> WmCtlError {
        WmCtlError::Win(err)
    }
}
