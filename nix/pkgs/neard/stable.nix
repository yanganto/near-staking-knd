{ pkgs, fenix }: pkgs.callPackage ./patch-stable.nix { inherit (fenix.packages) fromToolchainFile; }
