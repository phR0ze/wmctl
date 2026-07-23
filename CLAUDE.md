# CLAUDE.md

## What this is

`wmctl` is a Rust CLI for X11 window manipulation, implementing a subset of the EWMH/ICCCM specs
so it can work alongside any EWMH-compatible window manager (shaping and positioning windows in
ways the WM itself may not support natively). The repo is a Cargo workspace:

- `libwmctl/` — the library crate (`x11rb` is its only runtime dependency). Does the actual X11
  protocol work: connecting, reading/writing window properties, sending client messages.
- `src/` (crate `wmctl`) — the CLI binary. Thin `clap`-based wrapper around `libwmctl` that maps
  subcommands (`move`, `shape`, `place`, `static`, `list`, `info`) to library calls.

`libwmctl` is published and versioned independently on crates.io and is usable standalone (see
`libwmctl/examples/`); `wmctl` depends on the published version of `libwmctl` by default (the
path dependency in the root `Cargo.toml` is commented out — uncomment it when developing both
crates together in lockstep).

## Development environment

This is developed on NixOS. Use `nix develop` to enter the dev shell defined in `flake.nix`
before running any `cargo`/`rustc` commands — it provides the pinned Rust toolchain (`cargo`,
`rustc`, `rustfmt`, `clippy`, `rust-analyzer`) and `git` (needed by `build.rs`, see below). The
`nixpkgs` input is pinned to the same commit used in `~/Projects/nixos-config`, so update both
together if you bump it. Don't rely on any system-wide/global Rust install; if `cargo`/`rustc`
aren't on `PATH`, run `nix develop` first.

There is no automated test suite — verification is done manually against a running X11 session
via the CLI or the `libwmctl/examples/*.rs` binaries, since correctness depends on real window
manager behavior. When changing `libwmctl` internals, prefer exercising the relevant example
under an actual X session over trying to unit test X11 protocol interactions.

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


