use crate::WmCtlError;
use std::{convert, fmt};

/// Position provides a number of pre-defined positions on the screen to quickly and easily
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
    Static(i32, i32),
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
