{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.26.1";
  sha256 = "sha256-WoQtDdbFcvl6Wp5uv2tr/W/YYH8dyezF+LzSJ5oJcYY=";
  cargoSha256 = "sha256-kIcrdfTrIEVcOdDTHsfK75aEiNY3PN0W0S3V+r7vwnw=";
}
