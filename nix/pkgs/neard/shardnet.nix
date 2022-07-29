{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "2022-07-28";
  # compare with https://github.com/near/nearcore/releases/tag/shardnet
  rev = "c1b047b8187accbf6bd16539feb7bb60185bdc38";
  sha256 = "sha256-QokYs/ET2erO4J9aaaSJUpgMsZIyN/1GP5m9gSIlyS0=";
  cargoSha256 = "sha256-khTuIw2O6H71EyculFh0PmyzYSJwTz15jg7cl9OU8WU=";
  cargoBuildFlags = [
    "--features=shardnet"
  ];
}
