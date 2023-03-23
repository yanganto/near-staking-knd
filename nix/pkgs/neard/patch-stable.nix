{ pkgs, fromToolchainFile, neardPatches ? [ ], revisionNumber ? null }:
let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./stable-rust-toolchain.toml;
  toolchainChecksum = "sha256-S4dA7ne2IpFHG+EnjXfogmqwGyDFSRWFnJ8cy4KZr1k=";

  toolchain = fromToolchainFile {
    file = ./stable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.32.0";
  sha256 = "sha256-FBpHxb92cM59fRBxPpzOCaZ308yMnFES0mB/fgrkgWc=";
  cargoSha256 = "sha256-tKt98TpuibJsABr4r0Xi4SiCRlu7xj+XbkZHvMq0Zio=";
  inherit toolchainFile toolchainChecksum toolchain neardPatches revisionNumber;
}
