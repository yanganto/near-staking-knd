{ inputs
, ...
}:
{
  imports = [
    ./treefmt.nix
  ];
  perSystem =
    { self'
    , inputs'
    , config
    , system
    , pkgs
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
        neard = pkgs.callPackage ./neard/stable.nix {
          inherit (inputs') fenix;
        };
        neard-unstable = pkgs.callPackage ./neard/unstable.nix {
          inherit (inputs') fenix;
        };
        inherit (pkgs.callPackages ./near-cli/overrides.nix { }) near-cli;

        kneard = pkgs.callPackage ./kneard/stable.nix {
          inherit cargoLock;
        };
        kneard-unstable = pkgs.callPackage ./kneard/unstable.nix {
          inherit cargoLock;
        };
        kneard-mgr = pkgs.callPackage ./kneard-mgr.nix {
          inherit cargoLock;
          inherit (inputs'.nixos-anywhere.packages) nixos-anywhere;
          inherit (config.packages) neard;
        };

        near-staking-analytics = pkgs.callPackage ./near-staking-analytics {
          npmlock2nix = pkgs.callPackage inputs.npmlock2nix { };
          inherit (inputs) near-staking-ui;
        };

        # passthru as convenience for the CI.
        inherit (pkgs) nix-update;

        near-prometheus-exporter = pkgs.callPackage ./near-prometheus-exporter.nix { };

        default = self'.packages.kneard-mgr;
      }
      // (pkgs.lib.optionalAttrs (pkgs.system == "x86_64-linux") {
        near-bin = pkgs.callPackage ./neard/bin.nix { };
      });
    };
}
