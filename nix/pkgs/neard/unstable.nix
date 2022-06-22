{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.27.0-rc.5";
  sha256 = "sha256-JUBCDPNIyYLbvYoQqYMgNPF/k59eqArmFTbrN1ydggs=";
  cargoSha256 = "sha256-2OuhdZn+ek5NhM9so1FdZnYCsITrZy3FksPhbN2aN7w=";
}
