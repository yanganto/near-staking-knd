{ pkgs, fenix, lib }:

let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./unstable-rust-toolchain.toml;
  toolchainChecksum = "sha256-DzNEaW724O8/B8844tt5AVHmSjSQ3cmzlU4BP90oRlY=";

  toolchain = fenix.packages.fromToolchainFile {
    file = ./unstable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.31.0-rc.4";
  sha256 = "sha256-SZLMrUV0tSOsHlufM2Ycr5fswE3WJjjDmFcftfEH2nU=";
  cargoSha256 = "sha256-HRNsoHGqvArHBRIxGFlBZd362kDhNJt/X2Mr4r0jVQI=";
  inherit toolchainFile toolchainChecksum toolchain;
}
