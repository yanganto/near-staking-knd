{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.28.1";
  sha256 = "sha256-lAbVcmr8StAZAII++21xiBd4tRcdprefvcGzPLIjl74=";
  cargoSha256 = "sha256-4mqCLBudfsghguQ/c+XzhOAjHzd90fZcwq1UoQyJHeo=";
}
