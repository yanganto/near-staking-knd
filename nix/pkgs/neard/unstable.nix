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
  ver = "1.32.0-rc.2";
  sha256 = "sha256-n88pQCjyqscVTSLfr46w7gRT6uqb9iv2ln0h8NBuASc=";
  cargoSha256 = "sha256-r+3yncOjA3eYEqbxMrX7ykWZijsnBHSUNJSNvbrQ6FI=";
  inherit toolchainFile toolchainChecksum toolchain;
}
