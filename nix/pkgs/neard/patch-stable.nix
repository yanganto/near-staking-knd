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
  ver = "1.32.2";
  sha256 = "sha256-l85j9eDq7ZdOxEdhSaQTkWp1OndcwjtkDcUPK8SkSLE=";
  cargoSha256 = "sha256-/AUzlVP7/2p1oi5uTwMZce+G5iOb7qSDC9zVuN5Pg+M=";
  inherit toolchainFile toolchainChecksum toolchain neardPatches revisionNumber;
}
