{ pkgs, buildPackages, rustToolchain_1_63 }:

let
  generic = pkgs.callPackage ./generic.nix { };
  neardRustPlatform = pkgs.callPackage buildPackages.makeRustPlatform {
    rustc = rustToolchain_1_63.rustc;
    cargo = rustToolchain_1_63.cargo;
  };
in
generic {
  version = "1.29.0";
  sha256 = "sha256-TOZ6j4CaiOXmNn8kgVGP27SjvLDlGvabAA+PAEyFXIk=";
  cargoSha256 = "sha256-CUM597syeYh7dwrR3UR9d7njR5IKGu66b+RPoV3cR+4=";
  inherit neardRustPlatform;
}
