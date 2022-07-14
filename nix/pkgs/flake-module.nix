{ self, ... }:
{
  perSystem = { config, self', inputs', pkgs, ... }: {
    packages = {
      neard = pkgs.callPackage ./neard/stable.nix { };
      neard-unstable = pkgs.callPackage ./neard/unstable.nix { };
      neard-bin = pkgs.callPackage ./neard/bin.nix { };
      neard-shardnet = inputs'.nixpkgs-staging-next.legacyPackages.callPackage ./neard/shardnet.nix { };
      near-cli = pkgs.nodePackages.near-cli;

      kuutamod = pkgs.callPackage ./kuutamod.nix { };

      # passthru as convinience for the CI.
      nix-update = pkgs.nix-update;

      default = self'.packages.kuutamod;
    };
  };
}
