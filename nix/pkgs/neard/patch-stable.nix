{ pkgs, fromToolchainFile, neardPatches ? [ ], revisionNumber ? null }:
let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./stable-rust-toolchain.toml;
  toolchainChecksum = "sha256-JvgrOEGMM0N+6Vsws8nUq0W/PJPxkf5suZjgEtAzG6I=";

  toolchain = fromToolchainFile {
    file = ./stable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.33.0";
  sha256 = "sha256-lVH/QusAjUrWYDI/8EpSfH2skBlDgGco+E8LJBV9sIw=";
  cargoSha256 = "sha256-jW7XSQOcdUlLCu4MEZ7i6OVXDevmQoARWSNwr9wbH3c=";
  inherit toolchainFile toolchainChecksum toolchain neardPatches revisionNumber;
}
