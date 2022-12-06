{ pkgs, buildPackages, rustToolchain_1_63 }:

let
  generic = pkgs.callPackage ./generic.nix { };
  neardRustPlatform = pkgs.callPackage buildPackages.makeRustPlatform {
    rustc = rustToolchain_1_63.rustc;
    cargo = rustToolchain_1_63.cargo;
  };
in
generic {
  version = "1.29.3";
  sha256 = "sha256-Qbpp+ITWVFbigWLdSDHAo5JhHejEN2FknRIjcpcS2wY=";
  cargoSha256 = "sha256-JQp0pLF/5nSmPxStVVgyVXHkWLFxCxSF/HDogeC44do=";
  inherit neardRustPlatform;
}
