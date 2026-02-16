{
  description = "NGL - Nix Global Lookup";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            visidata
          ];
          buildInputs = with pkgs; [
            rustToolchain
            cargo
            rustc
            rust-analyzer
            rustfmt
            clippy

            # SQLite for rusqlite
            sqlite

            # We love seaORM!
            sea-orm-cli

            # OpenSSL for reqwest
            openssl
            pkg-config
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            export DATABASE_URL="sqlite://ngl.db?mode=rwc"
            export NGL_NIXPKGS_RELEASE="nixpkgs-26.05pre944764.2343bbb58f99"
            echo "NGL development environment"
            echo "Rust version: $(rustc --version)"
            alias ngl="$PWD/target/debug/ngl"
          '';
        };
      }
    );
}
