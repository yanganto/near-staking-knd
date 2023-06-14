{ self, ... }: {
  perSystem =
    { self'
    , inputs'
    , pkgs
    , config
    , ...
    }:
    let
      formatters = [
        # our meta-formatter
        config.treefmt.build.wrapper
        pkgs.clippy
      ];
    in
    {
      devShells.default = pkgs.stdenv.mkDerivation {
        name = "env";
        phases = [ "buildPhase" ];
        buildPhase = "touch $out";
        buildInputs =
          formatters
          ++ [
            # tasks and automation
            pkgs.just
            pkgs.jq
            self'.packages.near-cli
            pkgs.nix-update

            inputs'.nixos-anywhere.packages.nixos-anywhere

            # Benchmark tools
            #pkgs.fio
            #pkgs.numactl
            #pkgs.xmrig
            #pkgs.hwloc
            #(pkgs.inxi.override { withRecommends = true; })

            # for tests
            pkgs.mypy
            (pkgs.python3.withPackages (ps: [
              ps.pytest
              (ps.callPackage ./pkgs/remote-pdb.nix { })
            ]))

            # rust dev
            pkgs.rust-analyzer
            pkgs.cargo-watch
            pkgs.clippy
            pkgs.mold
            pkgs.rust-analyzer

            # kneard deps
            self'.packages.neard
            pkgs.consul
            pkgs.hivemind
          ]
          ++ self'.packages.kneard.buildInputs;
        CORE_CONTRACTS = self.inputs.core-contracts;
        inherit (self'.packages.kneard) nativeBuildInputs;
        NEARD_VERSION = "${self'.packages.neard.version}";
        passthru = {
          inherit formatters;
        };
      };
    };
}
