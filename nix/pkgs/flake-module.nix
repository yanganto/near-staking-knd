{ self
, fenix
, ...
}: {
  perSystem =
    { config
    , self'
    , inputs'
    , pkgs
    , ...
    }: {
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

        treefmt = self.inputs.treefmt-nix.lib.mkWrapper pkgs {
          # Used to find the project root
          projectRootFile = "flake.lock";

          programs.rustfmt.enable = true;

          settings.formatter = {
            nix = {
              command = "sh";
              options = [
                "-eucx"
                ''
                  # First deadnix
                  ${pkgs.lib.getExe pkgs.deadnix} --edit "$@"
                  # Then nixpkgs-fmt
                  ${pkgs.lib.getExe pkgs.nixpkgs-fmt} "$@"
                ''
                "--"
              ];
              includes = [ "*.nix" ];
              excludes = [ "nix/sources.nix" ];
            };
            shell = {
              command = "sh";
              options = [
                "-eucx"
                ''
                  # First shellcheck
                  ${pkgs.lib.getExe pkgs.shellcheck} --external-sources --source-path=SCRIPTDIR "$@"
                  # Then format
                  ${pkgs.lib.getExe pkgs.shfmt} -i 2 -s -w "$@"
                ''
                "--"
              ];
              includes = [ "*.sh" ];
            };

            python = {
              command = "sh";
              options = [
                "-eucx"
                ''
                  ${pkgs.lib.getExe pkgs.ruff} --fix "$@"
                  ${pkgs.lib.getExe pkgs.python3.pkgs.black} "$@"
                ''
                "--" # this argument is ignored by bash
              ];
              includes = [ "*.py" ];
            };
          };
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
