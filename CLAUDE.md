# CLAUDE.md

## What this is

`wmctl` is a Rust CLI for X11 window manipulation, implementing a subset of the EWMH/ICCCM specs
so it can work alongside any EWMH-compatible window manager (shaping and positioning windows in
ways the WM itself may not support natively).

## Development environment

This is developed on NixOS. Use `nix develop` to enter the dev shell defined in `flake.nix`
before running any `cargo`/`rustc`/`rustfmt`/`clippy`/`rust-analyzer` commands.

### Testing a `libwmctl` change against the CLI before publishing

When a change touches `libwmctl`, the CLI can't pick it up until the new version is published
to crates.io — but you can build/run the CLI against your local `libwmctl` first as a check:

1. Bump `version` in `libwmctl/Cargo.toml`.
2. In the root `Cargo.toml`, comment out the registry `libwmctl = "x.y.z"` line and uncomment
   `libwmctl = { path = "libwmctl" }`.
3. `cargo build`/`cargo run` as normal to verify the CLI works against the local crate.
4. Once satisfied, publish `libwmctl` to crates.io at the bumped version.
5. Switch the root `Cargo.toml` back: comment out the `path` dependency and point the registry
   dependency at the newly published version, so the CLI once again tracks the published crate
   rather than the in-repo copy.

Don't leave the CLI pointed at the path dependency long-term — it's a local build check, not the
intended default wiring.
