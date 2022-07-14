# This file is only for nix-update (https://github.com/Mic92/nix-update/) to function
{ ... }:
with import <nixpkgs> { };
{
  neard = pkgs.callPackage ./stable.nix { };
  neard-unstable = pkgs.callPackage ./unstable.nix { };
  neard-shardnet = pkgs.callPackage ./shardnet.nix { };
}
