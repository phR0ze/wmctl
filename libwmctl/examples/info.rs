use libwmctl::prelude::*;

fn main() {
    let wmctl = WmCtl::connect().unwrap();
    let (_, wm_name) = wmctl.winmgr().unwrap();
    let win = wmctl.active_win().unwrap();
    println!("X11 Information");
    println!("-----------------------------------------------------------------------");
    println!("Window Manager:    {}", wm_name);
    println!("Composite Manager: {}", wmctl.composite_manager().unwrap());
    println!("Root Window:       {}", wmctl.root());
    println!("Work area:         {}x{}", wmctl.work_width(), wmctl.work_height());
    println!("Screen Size:       {}x{}", wmctl.width(), wmctl.height());
    println!("Desktops:          {}", wmctl.desktops().unwrap());
    println!();
    println!("Active Window");
    println!("{:-<120}", "");

    println!("{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}", "ID", "DSK", "PID", "X", "Y", "W", "H", "BORDERS", "TYPE", "STATE", "CLASS", "NAME");

    let pid = wmctl.win_pid(win).unwrap_or(-1);
    let desktop = wmctl.win_desktop(win).unwrap_or(-1);
    let typ = wmctl.win_type(win).unwrap_or(WinType::Invalid);
    let states = wmctl.win_state(win).unwrap_or(vec![WinState::Invalid]);
    let (x, y, w, h) = wmctl.win_geometry(win).unwrap_or((0,0,0,0));
    let (l, r, t, b) = wmctl.win_borders(win).unwrap_or((0, 0, 0, 0));
    let class = wmctl.win_class(win).unwrap_or("".to_owned());
    let name = wmctl.win_name(win).unwrap_or("".to_owned());
    println!("{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}",
        format!("{:0>8}", win), format!("{:>2}", desktop), pid,
        format!("{:<4}", x), format!("{:<4}", y), format!("{:<4}", w), format!("{:<4}", h), 
        format!("{},{},{},{}", l, r, t, b),
        typ.to_string(), format!("{:?}", states), class, name);
}