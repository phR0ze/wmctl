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
Shape the active window using the pre-defined `Shape::Small` shape which is a quarter of the 
screen.

```rust
use libwmctl::prelude::*;

fn main() {
    active().shape(Shape::Small).place().unwrap();
}
```

### Move window
Move the active window to the bottom left corner of the screen using the pre-defined 
`Position::BottomLeft` position.

```rust
use libwmctl::prelude::*;

fn main() {
    active().pos(Position::BottomLeft).place().unwrap();
}
```

### Place window
Combine the shape and move into a single command by placing the window. First the window is shaped 
using the pre-defined `Shap::Small` shape then it is moved to the bottom left using the 
pre-defined `Position:BottomLeft` position in a single operation.

```rust
use libwmctl::prelude::*;

fn main() {
    active().shape(Shape::Small).pos(Position::BottomLeft).place().unwrap();
}
```

### Window Manager info
```rust
use libwmctl::prelude::*;
use prettytable::{format, Cell, Row, Table};

fn main() {
    let wm = info().unwrap();
    let win = active();

    println!("Window Manager Information");
    println!("-----------------------------------------------------------------------");
    println!("Window Manager: {}", wm.name);
    println!("Compositing:    {}", wm.compositing);
    println!("Root Window:    {}", wm.root_win_id);
    println!("Work area:      {}x{}", wm.work_area.0, wm.work_area.1);
    println!("Screen Size:    {}x{}", wm.screen_size.0, wm.screen_size.1);
    println!("Desktops:       {}", wm.desktops);
    println!("Active Window:  {}", win.id);
    println!();

    println!("Window Manager Supported Functions:");
    let mut table = Table::new();
    table.set_format(
        format::FormatBuilder::new()
            .separator(format::LinePosition::Top, format::LineSeparator::new('-', '+', '+', '+'))
            .separator(format::LinePosition::Title, format::LineSeparator::new('=', '+', '+', '+'))
            .padding(1, 1)
            .build(),
    );
    table.set_titles(Row::new(vec![Cell::new("NAME"), Cell::new("ID")]));

    // Sort atoms by name
    let mut atoms = wm.supported.iter().collect::<Vec<_>>();
    atoms.sort_by(|a, b| a.1.cmp(b.1));
    for atom in atoms.iter() {
        table.add_row(Row::new(vec![Cell::new(&atom.1), Cell::new(&atom.0.to_string())]));
    }
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
