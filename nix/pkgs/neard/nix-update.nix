# This file is only for nix-update (https://github.com/Mic92/nix-update/) to function
{ ... }:
with import <nixpkgs> { };
with import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") { };
{
  neard = pkgs.callPackage ./stable.nix {
    rustToolchain_1_63 = fenix.packages.toolchainOf {
      channel = "stable";
      date = "2022-08-11";
      sha256 = "sha256-KXx+ID0y4mg2B3LHp7IyaiMrdexF6octADnAtFIOjrY=";
    };
  };
  neard-unstable = pkgs.callPackage ./unstable.nix { };
}
