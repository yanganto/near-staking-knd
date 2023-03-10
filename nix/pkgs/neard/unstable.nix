{ pkgs, fenix, lib }:

let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./unstable-rust-toolchain.toml;
  toolchainChecksum = "sha256-S4dA7ne2IpFHG+EnjXfogmqwGyDFSRWFnJ8cy4KZr1k=";

  toolchain = fenix.packages.fromToolchainFile {
    file = ./unstable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.32.0-rc.1";
  sha256 = "sha256-wFNrDlBC8C3FSjTTXKCAIha+Y0Y0tB7FN/gDMI8LRsU=";
  cargoSha256 = "sha256-HSJ4LPYzOesj3K2OXoABD1zAFafu39XJjtBMcPB6a94=";
  inherit toolchainFile toolchainChecksum toolchain;
}
