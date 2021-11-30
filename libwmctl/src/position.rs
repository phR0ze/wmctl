use crate::WmCtlError;
use crate::PositionError;
use std::{fmt, convert::TryFrom};

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
}

impl TryFrom<&str> for Position {
    type Error = WmCtlError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_ref() {
            "center" => Ok(Position::Center),
            "left" => Ok(Position::Left),
            "right" => Ok(Position::Right),
            "top" => Ok(Position::Top),
            "bottom" => Ok(Position::Bottom),
            "topleft" => Ok(Position::TopLeft),
            "topright" => Ok(Position::TopRight),
            "bottomleft" => Ok(Position::BottomLeft),
            "bottomright" => Ok(Position::BottomRight),
            _ => Err(PositionError::Invalid(val.to_string()).into()),
        }
    }
}

// Implement format! support
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

impl TryFrom<String> for Position {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Position::try_from(val.as_str())
    }
}