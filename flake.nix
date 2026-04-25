{
  description = "doze - Modular and composable audio plugin abstraction layer for Rust";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rust = pkgs.rust-bin.selectLatestNightlyWith (
          toolchain:
          toolchain.minimal.override {
            extensions = [
              "cargo"
              "rust-src"
              "rustfmt"
              "clippy"
            ];
          }
        );

        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
          llvm
          rust
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs;

          packages = with pkgs; [
            rust-analyzer
          ];

          RUST_LOG = "debug";
          RUST_BACKTRACE = "1";
        };
      }
    );
}
