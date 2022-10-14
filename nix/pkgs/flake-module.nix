{ self, ... }:
{
  perSystem = { config, self', inputs', pkgs, ... }: {
    packages = {
      neard = pkgs.callPackage ./neard/stable.nix { };
      neard-unstable = pkgs.callPackage ./neard/unstable.nix { };
      inherit (pkgs.callPackages ./near-cli/overrides.nix { }) near-cli;

      kuutamod = pkgs.callPackage ./kuutamod.nix { };

      # passthru as convinience for the CI.
      nix-update = pkgs.nix-update;

      default = self'.packages.kuutamod;
    } // (pkgs.lib.optionalAttrs (pkgs.system == "x86_64-linux") {
      near-bin = pkgs.callPackage ./neard/bin.nix { };
    });
  };
}
