{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.30.0-rc.2";
  sha256 = "sha256-mTdhofXJWN7YYttZH5ualc9hFM5glv+UE7ArTTkB0ss=";
  cargoSha256 = "sha256-XelcSVYFiSYL/u3oA09kEBB3bl+PgYqDcfVy3MXt5L0=";
}
