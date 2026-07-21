# Standard Rust dev shell for wmctl
# - https://nixos.wiki/wiki/Rust
#
# wmctl is a pure Rust X11 client (x11rb) with no native/C dependencies, so this shell
# stays minimal: just the Rust toolchain plus git, which build.rs reads (.git/logs/HEAD)
# to embed the current commit hash into the binary.
#
# X11 only exists on Linux, so we hand-roll the system list instead of pulling in
# flake-utils for multi-system support.
{
  inputs = {
    # nixos-unstable from 2026.07.05
    nixpkgs.url = "github:nixos/nixpkgs/d407951447dcd00442e97087bf374aad70c04cea";
  };
  outputs = { nixpkgs, ... }: let
    systems = [ "x86_64-linux" "aarch64-linux" ];
    forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f (import nixpkgs { inherit system; }));
  in
  {
    # Run: `nix build`
    # Builds the wmctl workspace (CLI + libwmctl) and stores the binary in result/bin/
    packages = forAllSystems (pkgs: {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = "wmctl";
        version = "0.0.51";
        src = pkgs.lib.cleanSource ./.;
        cargoLock = { lockFile = ./Cargo.lock; };
      };
    });

    # Run: `nix develop`
    # Creates a development shell to work from
    devShells = forAllSystems (pkgs: {
      default = pkgs.mkShell {
        packages = with pkgs; [
          bashInteractive # Solve for normal shell operation
          rust-analyzer   # Rust Analyzer binary
        ];

        nativeBuildInputs = with pkgs; [
          cargo   # Rust build tooling
          rustc   # Rust compiler
          rustfmt # Formatter (rustfmt.toml at repo root)
          clippy  # Linter
          git     # build.rs reads .git/logs/HEAD for the embedded commit hash
        ];

        # Set the rust source path for rust-analyzer to be happy
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };
    });
  };
}
