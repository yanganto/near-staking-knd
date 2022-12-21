{ self
, fenix
, ...
}: {
  perSystem =
    { self'
    , inputs'
    , pkgs
    , ...
    }: let
      cargoLock = {
        lockFile = ../../Cargo.lock;
        outputHashes = {
          "format_serde_error-0.3.0" = "sha256-R4zD1dAfB8OmlfYUDsDjevMkjfIWGtwLRRYGGRvZ8F4=";
        };
      };
    in {
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

        kuutamod = pkgs.callPackage ./kuutamod.nix {
          inherit cargoLock;
        };
        kuutamo = pkgs.callPackage ./kuutamo.nix {
          inherit cargoLock;
        };

        treefmt = pkgs.callPackage ./treefmt.nix {
          inherit (self.inputs) treefmt-nix;
        };

        # passthru as convenience for the CI.
        nix-update = pkgs.nix-update;

        default = self'.packages.kuutamod;
      }
      // (pkgs.lib.optionalAttrs (pkgs.system == "x86_64-linux") {
        near-bin = pkgs.callPackage ./neard/bin.nix { };
      });
    };
}
