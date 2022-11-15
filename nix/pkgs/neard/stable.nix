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
  cargoSha256 = "sha256-WgBm8ko8pRuDxNFQ0hCLalpTox3TpHLmEqowv2Dr5/c=";
  inherit neardRustPlatform;
}
