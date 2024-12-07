{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnFree = true;
        };
        # 👇 new! note that it refers to the path ./rust-toolchain.toml
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile
          ./rust-toolchain.toml;
      in with pkgs; {
        devShells.default = mkShell {
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
              pkgs.lib.makeLibraryPath [
                pkgs.alsa-lib
                pkgs.udev
                pkgs.vulkan-loader
                pkgs.libxkbcommon
              ]
            }"'';

          # 👇 we can just use `rustToolchain` here:
          buildInputs = [
            rustToolchain
            rust-analyzer
            rustfmt
            cargo-edit
            cargo-watch
            pkg-config
            alsa-lib
            jack2

            lld
            clang

            udev
            #lutris
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            vulkan-tools
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers
            libjack2
            just
            bacon

          ];
        };
      });
}

