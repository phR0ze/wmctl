# libwmctl
[![license-badge](https://img.shields.io/crates/l/fungus.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/libwmctl.svg)](https://crates.io/crates/libwmctl)
[![Minimum rustc](https://img.shields.io/badge/rustc-1.30+-lightgray.svg)](https://github.com/phR0ze/gory#rustc-requirements)

***Rust X11 automation***

`libwmctl` implements the [Extended Window Manager Hints (EWMH) specification](https://specifications.freedesktop.org/wm-spec/latest/)
as a way to work along side EWMH compatible window managers as a companion. `libwmctl` provides the 
ability to precisely define how windows should be shaped and placed and can fill in gaps for window 
managers lacking some shaping or placement features. `libwmctl` exposes X11 details in a simple
consumable way opening the door to window manipulation beyond what your favorite EWMH window manager 
provides.

### Quick links
* [Usage](#usage)
  * [Shape window](#shape-window)
  * [Move window](#move-window)
  * [Place window](#place-window)
  * [Window Manager info](#window-manager-info)
* [Contribute](#contribute)
* [License](#license)
  * [Contribution](#contribution)
* [Backlog](#backlog)
* [Changelog](#changelog)

## Usage
This minimum rustc requirement is driven by the
[tracing\_subscriber](https://docs.rs/tracing-subscriber/0.2.15/tracing_subscriber) requirements

### Shape window
Shape the active window using the pre-defined `WinShape::Small` shape which is a quarter of the 
screen.

```rust
use libwmctl::prelude::*;

fn main() {
    WinOpt::new(None).shape(WinShape::Max).place().unwrap();
}
```

### Move window
Move the active window to the bottom left corner of the screen using the pre-defined 
`WinPosition::BottomLeft` position.

```rust
use libwmctl::prelude::*;

fn main() {
    WinOpt::new(None).pos(WinPosition::BottomLeft).place().unwrap();
}
```

### Place window
Combine the shape and move into a single command by placing the window. First the window is shaped 
using the pre-defined `WinShap::Small` shape then it is moved to the bottom left using the 
pre-defined `WinPosition:BottomLeft` position in a single operation.

```rust
use libwmctl::prelude::*;

fn main() {
    WinOpt::new(None).shape(WinShape::Small).pos(WinPosition::BottomLeft).place().unwrap();
}
```

### Window Manager info
```rust
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
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new("DSK"),
        Cell::new("PID"),
        Cell::new("X"),
        Cell::new("Y"),
        Cell::new("W"),
        Cell::new("H"),
        Cell::new("BORDERS"),
        Cell::new("TYPE"),
        Cell::new("STATE"),
        Cell::new("CLASS"),
        Cell::new("NAME"),
    ]));

    let pid = wmctl.win_pid(win).unwrap_or(-1);
    let desktop = wmctl.win_desktop(win).unwrap_or(-1);
    let typ = wmctl.win_type(win).unwrap_or(WinType::Invalid);
    let states = wmctl.win_state(win).unwrap_or(vec![WinState::Invalid]);
    let (x, y, w, h) = wmctl.win_geometry(win).unwrap_or((0, 0, 0, 0));
    let (l, r, t, b) = wmctl.win_borders(win).unwrap_or((0, 0, 0, 0));
    let class = wmctl.win_class(win).unwrap_or("".to_owned());
    let name = wmctl.win_name(win).unwrap_or("".to_owned());
    table.add_row(Row::new(vec![
        Cell::new(&win.to_string()),
        Cell::new(&format!("{:>2}", desktop)),
        Cell::new(&pid.to_string()),
        Cell::new(&x.to_string()),
        Cell::new(&y.to_string()),
        Cell::new(&w.to_string()),
        Cell::new(&h.to_string()),
        Cell::new(&format!("L{},R{},T{},B{}", l, r, t, b)),
        Cell::new(&typ.to_string()),
        Cell::new(&format!("{:?}", states)),
        Cell::new(&class),
        Cell::new(&name),
    ]));
    table.printstd();
}
```

## Contribute
Pull requests are always welcome. However understand that they will be evaluated purely on whether
or not the change fits with my goals/ideals for the project.

## License
This project is licensed under either of:
 * MIT license [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT
 * Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.

---

## Backlog

## Changelog
