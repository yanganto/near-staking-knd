{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.31.0-rc.2";
  sha256 = "sha256-5TldO83qosAGZmcawqzlZOJ+dWMP9V1f4byD0lLKYE0=";
  cargoSha256 = "sha256-W3BePyP1bXVlh4Aj4JEhB4hCC4qg89Cj3eYo4NTWaS4=";
}
