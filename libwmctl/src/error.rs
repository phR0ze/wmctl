use crate::PositionError;
use crate::ShapeError;
use crate::WinError;
use std::fmt;
use std::error::Error as StdError;

/// `WmCtlResult<T>` provides a simplified result type with a common error type
pub type WmCtlResult<T> = std::result::Result<T, WmCtlError>;

// An error indicating that something went wrong with a window operation
#[derive(Debug)]
pub enum WmCtlError {
    // A position error
    Position(PositionError),

    // A shape error
    Shape(ShapeError),

    // A window error
    Win(WinError),

    // std::str::Utf8Error
    Utf8(std::str::Utf8Error),

    // x11rb errors
    Connect(x11rb::errors::ConnectError),
    Connection(x11rb::errors::ConnectionError),
    Reply(x11rb::errors::ReplyError),
}
impl WmCtlError {
    /// Implemented directly on the `Error` type to reduce casting required
    pub fn is<T: StdError + 'static>(&self) -> bool {
        self.as_ref().is::<T>()
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_ref<T: StdError + 'static>(&self) -> Option<&T> {
        self.as_ref().downcast_ref::<T>()
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_mut<T: StdError + 'static>(&mut self) -> Option<&mut T> {
        self.as_mut().downcast_mut::<T>()
    }

    /// Implemented directly on the `Error` type to reduce casting required
    /// which allows for using as_ref to get the correct pass through.
    pub fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.as_ref().source()
    }
}
impl StdError for WmCtlError {}

impl fmt::Display for WmCtlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WmCtlError::Position(ref err) => write!(f, "{}", err),
            WmCtlError::Shape(ref err) => write!(f, "{}", err),
            WmCtlError::Win(ref err) => write!(f, "{}", err),
            WmCtlError::Utf8(ref err) => write!(f, "{}", err),
            WmCtlError::Connect(ref err) => write!(f, "{}", err),
            WmCtlError::Connection(ref err) => write!(f, "{}", err),
            WmCtlError::Reply(ref err) => write!(f, "{}", err),
        }
    }
}

impl AsRef<dyn StdError> for WmCtlError {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        match *self {
            WmCtlError::Position(ref err) => err,
            WmCtlError::Shape(ref err) => err,
            WmCtlError::Win(ref err) => err,
            WmCtlError::Utf8(ref err) => err,
            WmCtlError::Connect(ref err) => err,
            WmCtlError::Connection(ref err) => err,
            WmCtlError::Reply(ref err) => err,
        }
    }
}

impl AsMut<dyn StdError> for WmCtlError {
    fn as_mut(&mut self) -> &mut (dyn StdError + 'static) {
        match *self {
            WmCtlError::Position(ref mut err) => err,
            WmCtlError::Shape(ref mut err) => err,
            WmCtlError::Win(ref mut err) => err,
            WmCtlError::Utf8(ref mut err) => err,
            WmCtlError::Connect(ref mut err) => err,
            WmCtlError::Connection(ref mut err) => err,
            WmCtlError::Reply(ref mut err) => err,
        }
    }
}

impl From<std::str::Utf8Error> for WmCtlError {
    fn from(err: std::str::Utf8Error) -> WmCtlError {
        WmCtlError::Utf8(err)
    }
}

// x11rb errors
//--------------------------------------------------------------------------------------------------
impl From<x11rb::errors::ConnectError> for WmCtlError {
    fn from(err: x11rb::errors::ConnectError) -> WmCtlError {
        WmCtlError::Connect(err)
    }
}

impl From<x11rb::errors::ConnectionError> for WmCtlError {
    fn from(err: x11rb::errors::ConnectionError) -> WmCtlError {
        WmCtlError::Connection(err)
    }
}

impl From<x11rb::errors::ReplyError> for WmCtlError {
    fn from(err: x11rb::errors::ReplyError) -> WmCtlError {
        WmCtlError::Reply(err)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_errors() {
    }
}
