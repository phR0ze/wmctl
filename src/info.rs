use clap::ArgMatches;
use libwmctl::prelude::*;
use prettytable::{format, Cell, Row, Table};

use crate::utils;

/// Run the subcommand
///
/// ### Arguments
/// * `global` - the ArgMatches object for the global arguments
pub fn run(global: &ArgMatches) {
    let matches = global.subcommand_matches("info").unwrap();

    if let Some(matches) = matches.subcommand_matches("winmgr") {
        winmgr(matches.is_present("all"));
    } else {
        window(utils::get_window_id(global, true));
    }
}

pub fn winmgr(all: bool) {
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

    if all {
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
}

// Print out the window's properties
pub fn window(id: u32) {
    let wm = info().unwrap();
    let win = libwmctl::window(id);
    let parent = win.parent().unwrap();

    let (px, py, pw, ph) = parent.visual_geometry().unwrap();
    let (x, y, w, h) = win.geometry().unwrap();
    let (vx, vy, vw, vh) = win.visual_geometry().unwrap();
    let b = win.borders();
    let g = win.gtk_borders();

    println!("Window Information");
    println!("-----------------------------------------------------------------------");
    println!("Class:        {}", win.class().unwrap_or("".to_owned()));
    println!("Name:         {}", win.name().unwrap_or("".to_owned()));
    println!("PID:          {}", win.pid().unwrap_or(-1));
    println!("ID:           {}", win.id);
    println!("Parent:       {}", parent.id);
    println!("Parent Geom:  x: {}, y: {}, w: {}, h: {}", px, py, pw, ph);
    if parent.id != wm.root_win_id {
        let grand_parent = parent.parent().unwrap();
        println!(
            "Grand Parent: {} {}",
            grand_parent.id,
            if grand_parent.id == wm.root_win_id { "is root window" } else { "is not root window" }
        );
    }
    println!("Type:         {}", win.kind().unwrap_or(Kind::Invalid));
    println!("Desktop:      {}", win.desktop().unwrap_or(-1));
    println!("Win Geom:     x: {}, y: {}, w: {}, h: {}", x, y, w, h);
    println!("Visual Geom:  x: {}, y: {}, w: {}, h: {}", vx, vy, vw, vh);
    println!("WM Borders:   l: {}, r: {}, t: {}, b: {}", b.l, b.r, b.t, b.b);
    println!("GTK Borders:  l: {}, r: {}, t: {}, b: {}", g.l, g.r, g.t, g.b);
    println!("State:        {:?}", win.state().unwrap_or(vec![]));
    println!("Mapped:       {}", win.mapped().unwrap());
}
