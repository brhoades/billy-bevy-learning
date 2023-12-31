{
  description = "billy Bevy learning repo";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }: let
    pkgsFor = system: import nixpkgs {
      inherit system;
    }; in (flake-utils.lib.eachDefaultSystem (system: let
      pkgs = pkgsFor system;
      isLinux = with pkgs; lib.strings.hasInfix "linux" system;
    in {
      devShells.default = with pkgs; mkShell {
        buildInputs = [
          cargo
          rustc
          rust-analyzer
          clippy
          rustfmt
          bacon

          pkg-config
        ] ++ (lib.optionals pkgs.stdenv.isLinux [
          alsaLib
          udev
          xorg.libX11
        ])
        ++ (lib.optionals pkgs.stdenv.isDarwin (with darwin.apple_sdk; [
          libiconv
          frameworks.AppKit
        ]));

        LD_LIBRARY_PATH = lib.optional pkgs.stdenv.isLinux (lib.makeLibraryPath (with xorg; [
          libX11
          libXcursor
          libXrandr
          libXi
          vulkan-loader
        ]));
      };
    }));
}
