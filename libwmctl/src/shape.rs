use crate::WmCtlError;
use std::{fmt, convert::TryFrom};

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Square,
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
impl TryFrom<&str> for Shape {
    type Error = WmCtlError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_ref() {
            "square" => Ok(Shape::Square),
            _ => Err(ShapeError::Invalid(val.to_string()).into()),
        }
    }
}

// Convert from a String to a Shape
impl TryFrom<String> for Shape {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Shape::try_from(val.as_str())
    }
}

// Shape Error
// -------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ShapeError {
    Invalid(String),
}
impl std::error::Error for ShapeError {}
impl fmt::Display for ShapeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ShapeError::Invalid(ref err) => write!(f, "invalid shape was given: {}", err),
        }
    }
}

impl From<ShapeError> for WmCtlError {
    fn from(err: ShapeError) -> WmCtlError {
        WmCtlError::Shape(err)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_errors() {
    }
}
