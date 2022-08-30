{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "2022-08-25";
  # compare with https://github.com/near/stakewars-iii/blob/main/commit.md
  rev = "fe435d02c5ea497933c89d5e7d1703d9379b7e1f";
  sha256 = "sha256-LeQooMbMCKzd/xLaVnIRCXoI5Uc8lOhJX/jQVKt1+h0=";
  cargoSha256 = "sha256-qEuNUIulaWWxRq8ZsZvb6LXJcBxCPWUkkdydZsedAJg=";
  cargoBuildFlags = [
    "--features=shardnet"
  ];
}
