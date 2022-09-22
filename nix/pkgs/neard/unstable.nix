{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.29.0";
  sha256 = "sha256-TOZ6j4CaiOXmNn8kgVGP27SjvLDlGvabAA+PAEyFXIk=";
  cargoSha256 = "sha256-LFYWkQY7UcFg0aImfS3cWGKviRdG+gP9Vv2QUZgxtsg=";
}
