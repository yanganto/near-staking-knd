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
  ver = "1.34.0-rc.2";
  sha256 = "sha256-KMfd07fPu4LGskCceQjMzq3veZ50d+5Jv5n8ELQnQ8k=";
  cargoSha256 = "sha256-UUPa02XaMQ88z1qhjolLiTdFzMMyVw8kbKfxRdy131c=";
  inherit toolchainFile toolchainChecksum toolchain;
}
