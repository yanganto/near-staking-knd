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
  ver = "1.35.0-rc.1";
  sha256 = "sha256-gNYPs19Ot7x2w4Lp8p0H4p0iOg8eVmdfxCTsumPrRz0=";
  cargoSha256 = "sha256-IKbG5yYP7zWYqlirnMwPKKyr66QBm1cO8Ln7T/Gn93k=";
  inherit toolchainFile toolchainChecksum toolchain;
}
