{ pkgs, buildPackages, rustPlatform, rustPackages_1_64 }:

let
  generic = pkgs.callPackage ./generic.nix { };
  neardRustPlatform = pkgs.callPackage buildPackages.makeRustPlatform {
    rustc = rustPackages_1_64.rustc;
    cargo = rustPackages_1_64.cargo;
  };
in
generic {
  version = "1.29.0";
  sha256 = "sha256-TOZ6j4CaiOXmNn8kgVGP27SjvLDlGvabAA+PAEyFXIk=";
  cargoSha256 = "sha256-LFYWkQY7UcFg0aImfS3cWGKviRdG+gP9Vv2QUZgxtsg=";
  inherit neardRustPlatform;
}
