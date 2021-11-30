use crate::WmCtlError;
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

// Implement format! support
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

// Convert from &str to Postiion
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

// Convert from String to Postiion
impl TryFrom<String> for Position {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Position::try_from(val.as_str())
    }
}

// Position Error
// -------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PositionError {
    Invalid(String),
}
impl std::error::Error for PositionError {}
impl fmt::Display for PositionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PositionError::Invalid(ref err) => write!(f, "invalid position was given: {}", err),
        }
    }
}

impl From<PositionError> for WmCtlError {
    fn from(err: PositionError) -> WmCtlError {
        WmCtlError::Position(err)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_errors() {
    }
}
