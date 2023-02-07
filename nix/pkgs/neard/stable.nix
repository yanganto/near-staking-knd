{ pkgs, buildPackages }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.31.0";
  sha256 = "sha256-gjaMb7U85TESQLSEWzi1y743W7BzVPUPmGX1W++vuFs=";
  cargoSha256 = "sha256-fBWtYHZXNgUDk/ec76Of+M7tPJ5QXKaFubBX8R+v6wU=";
}
