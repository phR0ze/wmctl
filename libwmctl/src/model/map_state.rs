use std::fmt;
use x11rb::protocol::xproto;

use crate::{WmCtlError, WmCtlResult};

/// MapState provides an easy way to identify the differnt window map values
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum MapState {
    Unmapped,
    Unviewable,
    Viewable,
}

// Convert from u32 to state
impl MapState {
    pub fn from(val: u32) -> WmCtlResult<MapState> {
        if val == xproto::MapState::UNMAPPED.into() {
            Ok(MapState::Unmapped)
        } else if val == xproto::MapState::UNVIEWABLE.into() {
            Ok(MapState::Unviewable)
        } else if val == xproto::MapState::VIEWABLE.into() {
            Ok(MapState::Viewable)
        } else {
            Err(WmCtlError::InvalidWinMap(val).into())
        }
    }
}

// Implement format! support
impl fmt::Display for MapState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}
