{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay?ref=master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils?ref=main";
    };
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        targets = [ "x86_64-unknown-linux-gnu" "thumbv8m.main-none-eabihf" ];
        extensions = [ "rust-src" "llvm-tools-preview" "rust-analyzer" ];

        rustPackages = pkgs.rust-bin.nightly.latest.default.override {
          inherit targets extensions;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustPackages
            pkgs.nixpkgs-fmt
            pkgs.taplo
            pkgs.probe-rs
            pkgs.flip-link
            pkgs.cargo-binutils
          ];
        };
      }
    );
}
