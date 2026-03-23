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
    flake-utils.lib.eachSystem
      [
        "x86_64-linux"
      ]
      (
        system:
        let
          inherit (nixpkgs) lib;
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
                "rustc-dev"
                "rustfmt"
                "clippy"
                "rustc-codegen-cranelift-preview"
              ];
            }
          );

          nativeBuildInputs = with pkgs; [
            pkg-config
            clang
            mold
            llvm
            rust
          ];

          buildInputs = with pkgs; [
            pipewire
            alsa-lib
            zlib
          ];
        in
        {
          devShells.default = pkgs.mkShell {
            inherit nativeBuildInputs buildInputs;

            packages = with pkgs; [
              rust-analyzer
            ];

            LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

            RUST_LOG = "debug";
            RUST_BACKTRACE = "1";
          };
        }
      );
}
