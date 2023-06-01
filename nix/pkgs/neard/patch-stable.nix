{ pkgs, fromToolchainFile, neardPatches ? [ ], revisionNumber ? null }:
let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./stable-rust-toolchain.toml;
  toolchainChecksum = "sha256-4vetmUhTUsew5FODnjlnQYInzyLNyDwocGa4IvMk3DM=";

  toolchain = fromToolchainFile {
    file = ./stable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.34.0";
  sha256 = "sha256-9o6WjqmqDKl9VPpmVeANRo9U78xp71NuEs7Lp4c51KY=";
  cargoSha256 = "sha256-MwSg8CNrcQ3I0DBCQbiUjYDYFMKjR7Ssd3kJ7kan4/w=";
  inherit toolchainFile toolchainChecksum toolchain neardPatches revisionNumber;
}
