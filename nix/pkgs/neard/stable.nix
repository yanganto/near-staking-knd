{ pkgs, makeRustPlatform }:

let
  generic = pkgs.callPackage ./generic.nix { };
  rustPlatform = makeRustPlatform {
    inherit (pkgs.rust-bin.stable."1.65.0") rustc cargo;
  };
in
generic {
  version = "1.31.1";
  sha256 = "sha256-4Vuxt1nNQDahxtSUMrfktx76XRFEh+nWKJ1u0gYXsuU=";
  cargoSha256 = "sha256-8HmMutnuU2KoTuvw2SSaPUCfCR1unUcGA3y9Yz/kJss=";
  inherit rustPlatform;
  toolchain = pkgs.rust-bin.stable."1.65.0".minimal;
}
