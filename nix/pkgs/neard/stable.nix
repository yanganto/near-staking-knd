{ pkgs, buildPackages }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.30.0-hotfix-Jan29";
  rev = "036b3ca7fd30a92b77c9fd4101481f511243f8fd";
  sha256 = "sha256-KFuinUTkG5S9Mm8U+Gj1JmsWxBjjkQS1FISvFUv2tS4=";
  cargoSha256 = "sha256-xrvMwpJH6zav9SE5+8PrvoJqqT5AdkFkKfVubVKAYEw=";
}
