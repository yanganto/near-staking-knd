{ pkgs, fromToolchainFile, neardPatches ? [ ], revisionNumber ? null }:
let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./stable-rust-toolchain.toml;
  toolchainChecksum = "sha256-eMJethw5ZLrJHmoN2/l0bIyQjoTX1NsvalWSscTixpI=";

  toolchain = fromToolchainFile {
    file = ./stable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.35.0";
  sha256 = "sha256-uYyaj/VjBJjt+svQG8tkTPeBtBz5vO9ZgOryHrVI/40=";
  cargoSha256 = "sha256-sp6NI9nkrZzHOiuPpuhTp19/CDL/+oSPLSEYbJZc7xI=";
  owner = "kuutamolabs";
  rev = "cb16ee2";
  inherit toolchainFile toolchainChecksum toolchain neardPatches revisionNumber;
}
