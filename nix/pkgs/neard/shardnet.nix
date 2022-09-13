{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "2022-09-02";
  # compare with https://github.com/near/stakewars-iii/blob/main/commit.md
  rev = "1897d5144a7068e4c0d5764d8c9180563db2fe43";
  sha256 = "sha256-Oz1sg17mvkEE5lvYcii2zcOAEEn7ReustuwInGc1p70=";
  cargoSha256 = "sha256-2Bej/43cPThKfY4TICv2/y53kNt54MiJmdy4yE7fWDs=";
  cargoBuildFlags = [
    "--features=shardnet"
  ];
}
