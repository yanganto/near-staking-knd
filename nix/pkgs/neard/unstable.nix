{ pkgs, fenix, lib }:

let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./unstable-rust-toolchain.toml;
  toolchainChecksum = "sha256-eMJethw5ZLrJHmoN2/l0bIyQjoTX1NsvalWSscTixpI=";

  toolchain = fenix.packages.fromToolchainFile {
    file = ./unstable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.36.0-rc.1";
  sha256 = "sha256-6dHPEfg6MsCducA1rcHIFjdspBySE41XA0xti6yPcBU=";
  cargoSha256 = "sha256-v+wj54EYieEleexwrxIps3XzQ37xrUDfQLcmIrEhB94=";
  inherit toolchainFile toolchainChecksum toolchain;
}
