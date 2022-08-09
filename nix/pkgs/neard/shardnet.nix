{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "2022-08-04";
  # compare with https://github.com/near/stakewars-iii/blob/main/commit.md
  rev = "78ef2f55857d6118047efccf070ae0f7ddb232ea";
  sha256 = "sha256-4oP2AlsnLS/4iaSq7rkSp0yYtD92klSgSeMykUMGbZw=";
  cargoSha256 = "sha256-XFYDI0N+4UEaGL+DF22Lpj8vT6ti4rsaLvBFY9sbiIU=";
  cargoBuildFlags = [
    "--features=shardnet"
  ];
}
