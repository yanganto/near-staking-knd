{ self, fenix, ... }:
{
  perSystem = { config, self', inputs', pkgs, ... }: {
    packages = {
      neard = pkgs.callPackage ./neard/stable.nix {
        rustToolchain_1_63 = inputs'.fenix.packages.toolchainOf {
          channel = "stable";
          date = "2022-08-11";
          sha256 = "sha256-KXx+ID0y4mg2B3LHp7IyaiMrdexF6octADnAtFIOjrY=";
        };
      };
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
