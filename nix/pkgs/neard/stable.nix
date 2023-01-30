{ pkgs, buildPackages }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.30.0";
  rev = "2e27b0791de731b4ec31e15d5c896033f0315ebc";
  sha256 = "sha256-nY31ebYOHnBKvLPldQw1rCRxGZ6pcm01+UHHwCYv0WI=";
  cargoSha256 = "sha256-URRC63rBPjfopt/pwfyOwA5DMPT+Z1snItStg6z3CSI=";
}
