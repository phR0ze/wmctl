use std::collections::HashMap;

/// WinMgr provides information about the window manager and its environment.
pub struct Info {
    pub id: u32,
    pub name: String,
    pub compositing: bool,
    pub root_win_id: u32,
    pub work_area: (u32, u32),
    pub screen_size: (u32, u32),
    pub desktops: u32,
    pub supported: HashMap<u32, String>,
}
