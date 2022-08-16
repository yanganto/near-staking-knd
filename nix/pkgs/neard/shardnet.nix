{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "2022-08-16";
  # compare with https://github.com/near/stakewars-iii/blob/main/commit.md
  rev = "f7f0cb22e85e9c781a9c71df7dcb17f507ff6fde";
  sha256 = "sha256-kSHZ0xnba0finFFk4wKAYayllejCC14/IBAZOJqs/pM=";
  cargoSha256 = "sha256-NhLtWZckCIAYCsan6hH1K2upsoVp/AaeXkR+hHULNFw=";
  cargoBuildFlags = [
    "--features=shardnet"
  ];
}
