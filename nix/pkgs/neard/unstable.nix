{ pkgs, fenix, lib }:

let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./unstable-rust-toolchain.toml;
  toolchainChecksum = "sha256-ks0nMEGGXKrHnfv4Fku+vhQ7gx76ruv6Ij4fKZR3l78=";

  toolchain = fenix.packages.fromToolchainFile {
    file = ./unstable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.36.0-rc.1";
  sha256 = "sha256-6dHPEfg6MsCducA1rcHIFjdspBySE41XA0xti6yPcBU=";
  cargoSha256 = "sha256-FphV2P8zdm6R3cWKIjDvPikqHcFzGUc++LuSJrXWISQ=";
  inherit toolchainFile toolchainChecksum toolchain;
}
