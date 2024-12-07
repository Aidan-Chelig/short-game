
{ pkgs ? import <nixpkgs> {} }:

let

  fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") { };
in

  pkgs.mkShell {

    shellHook = ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
      pkgs.alsaLib
      pkgs.udev
      pkgs.vulkan-loader
    ]}"'';


    nativeBuildInputs = with pkgs; [
      pkg-config
    ];
    buildInputs = with pkgs; [

      (
        with fenix;
        combine (
          with default; [
            cargo
            clippy-preview
            latest.rust-src
            rust-analyzer
            rust-std
            rustc
            rustfmt-preview
          ]
          )
          )
          cargo-edit
          cargo-watch
          pkg-config
          alsaLib
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

    # # bevy-specific deps (from https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md)
  ];

}
