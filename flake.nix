{
  description = "A devShell for Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system;
          overlays = overlays;
        };
        derivation = pkgs.rustPlatform.buildRustPackage rec {
          buildInputs = with pkgs; [
            git
          ];
          name = "giga";
          pname = name;
          cargoLock.lockFile = ./Cargo.lock;
          src = ./.;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          nativeBuildInputs = [ ];

          buildInputs = [
            rust-analyzer
            (rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
              ];
            })
          ];

          shellHook = ''
            exec zsh
          '';
        };

        packages.default = derivation;
      }
    );
}
