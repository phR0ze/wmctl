use crate::WmCtlError;
use std::{convert, fmt};

/// Shape provides a number of pre-defined shapes to manipulate the window into, taking into
/// account borders and taskbars automatically.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Grow,
    Max,
    Halfw,
    Halfh,
    Small,
    Medium,
    Large,
    Shrink,
    Square,
    UnMax,
    Static(u32, u32),
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
impl convert::TryFrom<&str> for Shape {
    type Error = WmCtlError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_ref() {
            "grow" => Ok(Shape::Grow),
            "max" => Ok(Shape::Max),
            "halfw" => Ok(Shape::Halfw),
            "halfh" => Ok(Shape::Halfh),
            "small" => Ok(Shape::Small),
            "medium" => Ok(Shape::Medium),
            "large" => Ok(Shape::Large),
            "shrink" => Ok(Shape::Shrink),
            "unmax" => Ok(Shape::UnMax),
            _ => Err(WmCtlError::InvalidWinShape(val.to_string()).into()),
        }
    }
}

// Convert from a String to a Shape
impl convert::TryFrom<String> for Shape {
    type Error = WmCtlError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Shape::try_from(val.as_str())
    }
}
