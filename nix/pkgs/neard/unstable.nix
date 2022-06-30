{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.28.0-rc.1";
  sha256 = "sha256-Ev5/jg4hrUyNCqst8b0mKhC1YG5DTuqzedQH5UO98D8=";
  cargoSha256 = "sha256-XRMKc5DBDzSraaIINtH6+AekhuQtgczYP/935wVbOqc=";
}
