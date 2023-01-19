let
  moz_overlay = import (fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
    rustChannel = nixpkgs.rustChannelOf {
     channel = "stable";
  };
  rust = (rustChannel.rust.override {
    targets = [
      "wasm32-unknown-unknown" # required for the web-app
    ];
    extensions = ["rust-src" "rust-analysis"];
  });

in
with nixpkgs;
pkgs.mkShell {
  shellHook = ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
    pkgs.alsaLib
    pkgs.udev
    pkgs.vulkan-loader
  ]}"'';

  buildInputs = with nixpkgs; [
    cargo-edit
    cargo-watch
    rust

    lld
    clang

    # # bevy-specific deps (from https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md)
    pkgconfig
    udev
    lutris
    xlibsWrapper
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    vulkan-tools
    vulkan-headers
    vulkan-loader
    vulkan-validation-layers
    libjack2
  ];

}
