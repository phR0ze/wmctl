use crate::{WmCtlError, WmCtlResult};
use std::fmt;
use x11rb::protocol::xproto;

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
