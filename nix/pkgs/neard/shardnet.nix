{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "2022-07-14";
  rev = "c00821fbbb5ac68b3b17ab436e16dea093a1cb45";
  sha256 = "sha256-H+GMqcd3QgXVn2Tb0g0I9+Ljz6yM8faayveAJwLDiqo=";
  cargoSha256 = "sha256-J5uHt8lpUQKjnZlgGAA8szx1dxglqwiXbh9zqbxwNi0=";
  cargoBuildFlags = [
    "--features=shardnet"
  ];
}
