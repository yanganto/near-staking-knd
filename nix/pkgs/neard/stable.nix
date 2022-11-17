{ pkgs, buildPackages, rustToolchain_1_63 }:

let
  generic = pkgs.callPackage ./generic.nix { };
  neardRustPlatform = pkgs.callPackage buildPackages.makeRustPlatform {
    rustc = rustToolchain_1_63.rustc;
    cargo = rustToolchain_1_63.cargo;
  };
in
generic {
  version = "1.29.1";
  sha256 = "sha256-TmmGLrDpNOfadOIwmG7XRgI89XQjaqIavxCEE2plumc=";
  cargoSha256 = "sha256-U/oxgsekDzRJF95ln+psJyv8BXTyOXeeM4C/3R/JASw=";
  inherit neardRustPlatform;
}
