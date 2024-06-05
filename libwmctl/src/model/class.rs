use crate::{WmCtlError, WmCtlResult};
use std::fmt;
use x11rb::protocol::xproto;

/// WinClass provides a easy way to identify the different window class types
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Class {
    CopyFromParent,
    InputOnly,
    InputOutput,
}

// Convert from u32 to Class
impl Class {
    pub fn from(val: u32) -> WmCtlResult<Class> {
        if val == xproto::WindowClass::COPY_FROM_PARENT.into() {
            Ok(Class::CopyFromParent)
        } else if val == xproto::WindowClass::INPUT_ONLY.into() {
            Ok(Class::InputOnly)
        } else if val == xproto::WindowClass::INPUT_OUTPUT.into() {
            Ok(Class::InputOutput)
        } else {
            Err(WmCtlError::InvalidWinClass(val).into())
        }
    }
}

// Implement format! support
impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}
