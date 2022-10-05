{ self, ... }:
{
  perSystem = { config, self', inputs', pkgs, ... }: {
    packages = {
      neard = pkgs.callPackage ./neard/stable.nix { };
      neard-unstable = pkgs.callPackage ./neard/unstable.nix { };
      neard-bin = pkgs.callPackage ./neard/bin.nix { };
      inherit (pkgs.callPackages ./near-cli/overrides.nix { }) near-cli;

      kuutamod = pkgs.callPackage ./kuutamod.nix { };

      db_bench = pkgs.callPackage ./db_bench.nix { };

      # passthru as convinience for the CI.
      nix-update = pkgs.nix-update;

      default = self'.packages.kuutamod;
    };
  };
}
