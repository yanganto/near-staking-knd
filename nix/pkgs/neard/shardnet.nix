{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "2022-07-28";
  # compare with https://github.com/near/nearcore/releases/tag/shardnet
  rev = "c1b047b8187accbf6bd16539feb7bb60185bdc38";
  sha256 = "sha256-QokYs/ET2erO4J9aaaSJUpgMsZIyN/1GP5m9gSIlyS0=";
  cargoSha256 = "sha256-zKDrrz9e66AY/GslohM7beAGz6utCnMsheI+L2h9PTU=";
  cargoBuildFlags = [
    "--features=shardnet"
  ];
}
