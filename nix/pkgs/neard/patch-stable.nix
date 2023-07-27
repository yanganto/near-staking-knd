{ pkgs, fromToolchainFile, neardPatches ? [ ], revisionNumber ? null }:
let
  generic = pkgs.callPackage ./generic.nix { };
  toolchainFile = ./stable-rust-toolchain.toml;
  toolchainChecksum = "sha256-eMJethw5ZLrJHmoN2/l0bIyQjoTX1NsvalWSscTixpI=";

  toolchain = fromToolchainFile {
    file = ./stable-rust-toolchain.toml;
    sha256 = toolchainChecksum;
  };
in
generic {
  ver = "1.35.0";
  sha256 = "sha256-0pRgqtm4a3FW7ww2wFZa6rAQj26JF/YsNHIvwgg4LZU=";
  cargoSha256 = "sha256-EW52V5TZR4qi/6Q0ML16kTG1qecWQTV0CHyhPIgQiho=";
  owner = "kuutamolabs";
  rev = "0544fd1";
  inherit toolchainFile toolchainChecksum toolchain neardPatches revisionNumber;
}
