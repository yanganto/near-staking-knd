{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.28.0";
  sha256 = "sha256-DRVlD74XTYgy3GeUd/7OIl2aie8nEJLmrmmkwPRkrA8=";
  cargoSha256 = "sha256-vHwN+vPNIJH/8TH4ERXGPsok1JqhhGwaBrJGqtNb6KI=";
}
