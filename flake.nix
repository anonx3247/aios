{
  description = "AIOS development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Node.js ecosystem
            pkgs.nodejs_22
            pkgs.nodePackages.pnpm

            # Rust ecosystem
            pkgs.cargo
            pkgs.rustc
            pkgs.rust-analyzer
            pkgs.rustfmt
            pkgs.clippy

            # Utilities
            pkgs.git
          ];

          shellHook = ''
            echo "AIOS development environment loaded"
            echo "Node: $(node --version)"
            echo "pnpm: $(pnpm --version)"
            echo "Rust: $(rustc --version)"
          '';

          # Environment variables for Tauri on macOS
          WEBKIT_DISABLE_COMPOSITING_MODE = "1";
        };
      }
    );
}
