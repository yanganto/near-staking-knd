{ pkgs, rustPlatform }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.29.1";
  sha256 = "sha256-TmmGLrDpNOfadOIwmG7XRgI89XQjaqIavxCEE2plumc=";
  cargoSha256 = "sha256-I/bcn3BYzk26cHWz9e1PuEz7hUjbvIw1R9YAE9cNaEs=";
  neardRustPlatform = rustPlatform; # unstable using the latest stable rust
}
