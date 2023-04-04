{ pkgs, cargoLock, enableLint ? false }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  inherit cargoLock;
  inherit enableLint;
}
