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
  ver = "1.32.1";
  sha256 = "sha256-TlgEjTQmG8qst+xy0ES1dGXyJNIytFq/f2S6c8eM63U=";
  cargoSha256 = "sha256-m9kvFsQeSJaHHQ074kszDhp8M9DaqUKDk3oV6STpcnE=";
  inherit toolchainFile toolchainChecksum toolchain neardPatches revisionNumber;
}
