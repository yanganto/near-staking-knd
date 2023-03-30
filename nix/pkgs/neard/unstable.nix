{ pkgs, fenix, lib }:

let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./unstable-rust-toolchain.toml;
  toolchainChecksum = "sha256-JvgrOEGMM0N+6Vsws8nUq0W/PJPxkf5suZjgEtAzG6I=";

  toolchain = fenix.packages.fromToolchainFile {
    file = ./unstable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.33.0-rc.1";
  sha256 = "sha256-v7NBoTUufIxHTqC2E9X2lydRXi0nFYIT/IEdYjPUdCs=";
  cargoSha256 = "sha256-Tdhn7Q3vuUbS7CU217YxEXclotUiXOnvw9A34zUuT58=";
  inherit toolchainFile toolchainChecksum toolchain;
}
