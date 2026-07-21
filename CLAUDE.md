# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

## Development environment

This is developed on NixOS. Use `nix develop` to enter the dev shell defined in `flake.nix`
before running any `cargo`/`rustc` commands — it provides the pinned Rust toolchain (`cargo`,
`rustc`, `rustfmt`, `clippy`, `rust-analyzer`) and `git` (needed by `build.rs`, see below). The
`nixpkgs` input is pinned to the same commit used in `~/Projects/nixos-config`, so update both
together if you bump it. Don't rely on any system-wide/global Rust install; if `cargo`/`rustc`
aren't on `PATH`, run `nix develop` first.

## Common commands

All of the below assume you're inside `nix develop` (or prefix with `nix develop -c <cmd>`).

```bash
# Build everything (workspace)
cargo build

# Build/run the CLI
cargo run -- shape small
cargo run -- move bottom-left
cargo run -- place halfw top-left
cargo run -- list -a
cargo run -- info winmgr -a

# Run a libwmctl example directly against a live X11 session
cargo run -p libwmctl --example list
cargo run -p libwmctl --example win_info
cargo run -p libwmctl --example winmgr_info
cargo run -p libwmctl --example place
cargo run -p libwmctl --example shape
cargo run -p libwmctl --example position
cargo run -p libwmctl --example properties
cargo run -p libwmctl --example static

# Format (rustfmt.toml at repo root defines the style)
cargo fmt --all

# Lint
cargo clippy --all
```

There is no automated test suite — verification is done manually against a running X11 session
via the CLI or the `libwmctl/examples/*.rs` binaries, since correctness depends on real window
manager behavior. When changing `libwmctl` internals, prefer exercising the relevant example
under an actual X session over trying to unit test X11 protocol interactions.

## Architecture

### libwmctl entry points (`libwmctl/src/lib.rs`)

Everything funnels through a single lazily-initialized singleton:

```rust
fn WM() -> &'static RwLock<WinMgr>   // one X11 connection for the process lifetime
```

Public free functions (`active()`, `window(id)`, `windows(hidden)`, `info()`, `first_by_class()`)
are the only way callers construct `Window`/`Info` values — there's no way to open a second
independent connection. `prelude::*` re-exports everything needed for typical usage.

### Two-layer split inside libwmctl

- `winmgr.rs` (`pub(crate) struct WinMgr`) — owns the actual `x11rb` `RustConnection`, the atom
  cache, and all raw property get/set and client-message-send logic against the root window and
  individual windows. This is where EWMH/ICCCM protocol details live (see the module doc-comment
  at the top of `winmgr.rs` for spec references).
- `window.rs` (`pub struct Window`) — the ergonomic per-window handle exposed to consumers
  (`pid()`, `name()`, `class()`, `kind()`, `state()`, `shape()`, `pos()`, `place()`, etc). Nearly
  every method is a thin delegation to `WM().read().unwrap().window_*(self.id)`. `Window` also
  accumulates a pending `shape`/`pos` directive via a builder-style API before `place()` commits
  both in one combined move-resize operation.

When adding a new capability, the raw X11 property/message work goes in `winmgr.rs`; the
consumer-facing method goes in `window.rs` (or as a free function in `lib.rs` if it's not
per-window, like WM-level `info()`).

### Model types (`libwmctl/src/model/`)

Pure data/enum types with no X11 dependencies, each in its own file and re-exported flat from
`model/mod.rs`: `Shape`, `Position`, `Gravity`, `State`, `Kind`, `MapState`, `Property`, `Info`,
plus shared helpers `Border` and `Rect` (declared directly in `model/mod.rs`). `Shape` and
`Position` both implement `TryFrom<&str>`/`TryFrom<String>` to parse CLI string values (e.g.
`"halfw"` → `Shape::Halfw`, `"bottom-left"` → `Position::BottomLeft`) and both support a `Static`
variant for literal pixel values (`static` CLI subcommand). Adding a new named shape/position
means: add the enum variant, add its string parse arm, add its layout math in `winmgr.rs`, and
add it to the `--value_names` list for the relevant `clap` subcommand in `src/main.rs`.

### Errors (`libwmctl/src/error.rs`)

`WmCtlResult<T> = Result<T, ErrorWrapper>`. `ErrorWrapper` wraps both `libwmctl`'s own
`WmCtlError` enum (domain errors like `InvalidWinShape`, `PropertyNotFound`) and upstream
`x11rb`/`Utf8Error` errors behind one type, with `downcast_ref`/`is` helpers so callers can
inspect the underlying cause without matching on every variant. The CLI (`src/main.rs`) instead
uses `witcher` for its own top-level error handling/backtraces — the two error strategies are
independent (`witcher` at the CLI boundary, `ErrorWrapper` inside the library).

### CLI (`src/main.rs`)

Single `clap` (v2, builder API) app defined in one large `App::new(...)` chain; each subcommand's
`long_about` doubles as its `--help` text with worked examples, so keep them in sync with actual
behavior when changing flags. Dispatch after parsing is a flat if/else chain per subcommand into
`src/info.rs`, `src/list.rs`, `src/place.rs` (which also handles bare `move`/`shape`/`static`).
`src/utils.rs` holds small CLI-only helpers shared across those three.

### Versioning / git hooks

`.githooks/pre-commit` and `.githooks/commit-msg` auto-increment the patch version in both
`Cargo.toml` and `libwmctl/Cargo.toml` on every commit and prepend the resulting version to the
commit message — this only takes effect if `core.hooksPath` is pointed at `.githooks/` locally.
Because of this, don't hand-edit the `version` fields as part of a normal change; they're
generated by the commit hook.

### Packaging

`pkg/wmctl.nix` and `config/package/PKGBUILD` are reference packaging scripts (Nix and Arch
`PKGBUILD` respectively) meant to be copied into an external package repo/config, not built from
in-tree as part of normal development.
