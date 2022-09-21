{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.29.0-rc.4";
  sha256 = "sha256-LyvI3gn9alD3+LmUMPLreQ0Bm8P+RjF96tGVX3B2epk=";
  cargoSha256 = "sha256-4CHQN0jBErlU64+e6V4QUCO5qQ1ajPNaxvxMCheLO8Y=";
}
