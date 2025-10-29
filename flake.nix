{
  description = "Trialogue - A Rust project using Bevy ECS, glam, and wgpu";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Use the Rust toolchain specified in your project (stable by default)
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Native dependencies for wgpu and graphics
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        buildInputs = with pkgs; [
          # wgpu/graphics dependencies
          vulkan-loader
          vulkan-headers
          vulkan-validation-layers

          # X11/Wayland dependencies
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          wayland
          libxkbcommon

          # Additional graphics libs
          mesa
        ];

        # Library path for runtime linking
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          packages = with pkgs; [
            rustToolchain
            cargo
            rustc
            clippy
            rustfmt
          ];

          shellHook = ''
            export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:$LD_LIBRARY_PATH"
            export RUST_BACKTRACE=1
            echo "Rust development environment loaded"
            echo "Rust version: $(rustc --version)"
          '';
        };

        # Optional: build the package
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "trialogue";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;
        };
      }
    );
}
