{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.29.0-rc.3";
  sha256 = "sha256-7cUri06ZVMzXEe3VQfYSD1igLoqo21gShPsgj/ZtatQ=";
  cargoSha256 = "sha256-+7hxw4YkICLJdatKSnoISSrD7InAo47Y4SYUOs4+uAU=";
}
