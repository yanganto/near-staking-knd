{ pkgs, fenix, lib }:

let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./stable-rust-toolchain.toml;
  toolchainChecksum = "sha256-DzNEaW724O8/B8844tt5AVHmSjSQ3cmzlU4BP90oRlY=";

  toolchain = fenix.packages.fromToolchainFile {
    file = ./stable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  version = "1.31.1";
  sha256 = "sha256-4Vuxt1nNQDahxtSUMrfktx76XRFEh+nWKJ1u0gYXsuU=";
  cargoSha256 = "sha256-8HmMutnuU2KoTuvw2SSaPUCfCR1unUcGA3y9Yz/kJss=";
  inherit toolchainFile toolchainChecksum toolchain;
}
