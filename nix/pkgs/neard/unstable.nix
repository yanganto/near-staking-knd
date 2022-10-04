{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.30.0-rc.1";
  sha256 = "sha256-mEx6ANle56YOLCbESxhh2wEu2qioQxH1Icpw9c19fdU=";
  cargoSha256 = "sha256-ZQcfh2ZgV9wFaIu4tDOGWrBysJSJUl01RMYF6YhRTc4=";
}
