{ pkgs, buildPackages, rustToolchain_1_63 }:

let
  generic = pkgs.callPackage ./generic.nix { };
  neardRustPlatform = pkgs.callPackage buildPackages.makeRustPlatform {
    rustc = rustToolchain_1_63.rustc;
    cargo = rustToolchain_1_63.cargo;
  };
in
generic {
  version = "1.30.0";
  sha256 = "sha256-Co8896RojUf/R8ZiRn7zSO1AWH7x5rYom6TbGohH1KM=";
  cargoSha256 = "sha256-URRC63rBPjfopt/pwfyOwA5DMPT+Z1snItStg6z3CSI=";
  inherit neardRustPlatform;
}
