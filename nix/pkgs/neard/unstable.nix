{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.31.0-rc.1";
  sha256 = "sha256-H5L2CUpou1lIqX0IkPNoM0IyUiL4Y2sIDFCqjdB41RU=";
  cargoSha256 = "sha256-ApG9r8XL9LDSKDC3Ul2ssnAfncYt6LhVWX5XtfcyijE=";
}
