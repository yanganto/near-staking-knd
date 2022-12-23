{ pkgs, buildPackages }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.30.0";
  sha256 = "sha256-Co8896RojUf/R8ZiRn7zSO1AWH7x5rYom6TbGohH1KM=";
  cargoSha256 = "sha256-URRC63rBPjfopt/pwfyOwA5DMPT+Z1snItStg6z3CSI=";
}
