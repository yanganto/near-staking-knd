{ self, lib, ... }:

{
  perSystem = { pkgs, self', ... }:
    let
      generated-flake = pkgs.runCommand "generated-flake" { } ''
        cp -r ${./test-config.toml} test-config.toml
        cp -r ${./validator_key.json} validator_key.json
        cp -r ${./node_key.json} node_key.json
        ${lib.getExe self'.packages.kneard-mgr} --config test-config.toml generate-config "$out"
      '';
    in
    {
      checks = lib.optionalAttrs pkgs.stdenv.isLinux {
        kneard = import ./kneard.nix {
          inherit self pkgs;
        };
        neard = import ./neard.nix {
          inherit self pkgs;
        };
        monitoring = import ./monitoring.nix {
          inherit self pkgs;
        };
        kneard-mgr = pkgs.callPackage ./kneard-mgr.nix {
          inherit self pkgs;
        };

        generated-flake-is-same = pkgs.runCommand "generated-flake-is-same" { } ''
          if ! diff -Naur "${generated-flake}" "${./test-flake}"; then
            echo "Generated configuration in ./test-flake is no longer up-to-date!!" >&2
            echo "run the following command:" >&2
            echo "$ just ./nix/modules/tests/generate-test-flake" >&2
            exit 1
          fi
          touch $out
        '';
      };
    };
  flake = {
    nixosModules.qemu-test-profile = { modulesPath, ... }: {
      imports = [
        (modulesPath + "/testing/test-instrumentation.nix")
        (modulesPath + "/profiles/qemu-guest.nix")
      ];
      environment.etc."system-info.toml".text = ''
        git_sha = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        git_commit_date = "20230424000000"
      '';
    };
  } // import ./test-flake/configurations.nix {
    near-staking-knd = self;
  };
}
