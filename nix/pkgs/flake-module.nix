{ inputs
, ...
}: {
  imports = [
    ./treefmt.nix
  ];
  perSystem =
    { self'
    , inputs'
    , pkgs
    , config
    , ...
    }:
    let
      cargoLock = {
        lockFile = ../../Cargo.lock;
        outputHashes = {
          "format_serde_error-0.3.0" = "sha256-R4zD1dAfB8OmlfYUDsDjevMkjfIWGtwLRRYGGRvZ8F4=";
        };
      };
    in
    {
      packages = {
        neard = pkgs.callPackage ./neard/stable.nix { };
        neard-unstable = pkgs.callPackage ./neard/unstable.nix { };
        inherit (pkgs.callPackages ./near-cli/overrides.nix { }) near-cli;

        kuutamod = pkgs.callPackage ./kuutamod.nix {
          inherit cargoLock;
        };
        kuutamo = pkgs.callPackage ./kuutamo.nix {
          inherit cargoLock;
          inherit (inputs'.nixos-remote.packages) nixos-remote;
          inherit (config.packages) neard;
        };

        near-staking-analytics = pkgs.callPackage ./near-staking-analytics {
          npmlock2nix = pkgs.callPackage inputs.npmlock2nix { };
        };

        # passthru as convenience for the CI.
        inherit (pkgs) nix-update;

        default = self'.packages.kuutamo;
      }
      // (pkgs.lib.optionalAttrs (pkgs.system == "x86_64-linux") {
        near-bin = pkgs.callPackage ./neard/bin.nix { };
      });
    };
}
