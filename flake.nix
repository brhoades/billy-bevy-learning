{
  description = "billy Bevy learning repo";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }: let
    pkgsFor = system: import nixpkgs {
      inherit system;
    }; in (flake-utils.lib.eachDefaultSystem (system: {
      devShells.default = with (pkgsFor system); mkShell {
        buildInputs = [
          cargo
          rustc
          rust-analyzer
          clippy
          rustfmt
          bacon

          pkg-config
          alsaLib
          udev
          xorg.libX11
        ];

        LD_LIBRARY_PATH = lib.makeLibraryPath (with xorg; [
          libX11
          libXcursor
          libXrandr
          libXi
          vulkan-loader
        ]);
      };
    }));
}
