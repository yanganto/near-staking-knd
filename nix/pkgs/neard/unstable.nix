{ pkgs, fenix, lib }:

let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./unstable-rust-toolchain.toml;
  toolchainChecksum = "sha256-4vetmUhTUsew5FODnjlnQYInzyLNyDwocGa4IvMk3DM=";

  toolchain = fenix.packages.fromToolchainFile {
    file = ./unstable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.34.0-rc.1";
  sha256 = "sha256-pQDXhGjOppdmLSMEUQdTiV9Yr3MNdnBXmkLHzPiBtvE=";
  cargoSha256 = "sha256-OX/hIjpZf5LJMH6FA2I6URcLSirFuRR2KommNeZWdN4=";
  inherit toolchainFile toolchainChecksum toolchain;
}
