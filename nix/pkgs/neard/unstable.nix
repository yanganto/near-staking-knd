{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.28.0-rc.2";
  sha256 = "sha256-9jlcEJ+sYHB/gXyIDDK6vmq7CHlbItUxu4+q54w4czQ=";
  cargoSha256 = "sha256-ntfJUoJOi6QFiWm6k65MQoXGFRPod1yg3QlJ7eqyLG8=";
}
