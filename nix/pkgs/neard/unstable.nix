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
  ver = "1.36.0-rc.2";
  sha256 = "sha256-tDtSz1w+v5ux02hyAV/CETY1OJEFJYIDGGiut3MonMo=";
  cargoSha256 = "sha256-9eRACStaKwPeFbXRCNxx7EMyS/A9iR5N6aU1Fcs+lgE=";
  inherit toolchainFile toolchainChecksum toolchain;
}
