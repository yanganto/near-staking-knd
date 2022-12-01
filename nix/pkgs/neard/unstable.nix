{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.30.0-rc.5";
  sha256 = "sha256-4epgsk1VCqSsTncbwCuxStJuNzi+hUS/OeLVtejhcfQ=";
  cargoSha256 = "sha256-1joVE8+JYLbF9NjUmN9ciBP5bCS+wH9TnplwKlg/J1s=";
}
