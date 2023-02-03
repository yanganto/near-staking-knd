{ pkgs, buildPackages }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.30.1";
  sha256 = "sha256-VjvHCiWjsx5Y7xxqck/O9gSNrL8mxCTosLwLqC85ywY=";
  cargoSha256 = "sha256-9ZNrDqLW6tFwF3X2mysoxNw5PQojyglUDiVNYRvIiBE=";
}
