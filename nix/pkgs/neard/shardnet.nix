{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "2022-08-03";
  # compare with https://github.com/near/stakewars-iii/blob/main/commit.md
  rev = "68bfa84ed1455f891032434d37ccad696e91e4f5";
  sha256 = "sha256-HBYt7B7Ex0qQ3BpDWFfb/X7leTQ12luhZp6KR0kaVeE=";
  cargoSha256 = "sha256-fIAm8wCIE16ypc5Jav5iIwa2+yMg8N8yoeyyX0BRfMI=";
  cargoBuildFlags = [
    "--features=shardnet"
  ];
}
