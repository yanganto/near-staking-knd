{ pkgs, buildPackages }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.31.0";
  sha256 = "sha256-gjaMb7U85TESQLSEWzi1y743W7BzVPUPmGX1W++vuFs=";
  cargoSha256 = "sha256-uU+pJp5ZD540HVFf39TGMyQMCUQgAoZoUiR75orSyL0=";
}
