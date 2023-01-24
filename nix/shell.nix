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
      stdenv' = if pkgs.stdenv.hostPlatform.isGnu then pkgs.fastStdenv else pkgs.stdenv;
    in
    {
      devShells.default = stdenv'.mkDerivation {
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

            inputs'.nixos-remote.packages.nixos-remote

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

            # kuutamod deps
            self'.packages.neard
            pkgs.consul
            pkgs.hivemind
            (pkgs.writeShellScriptBin "local-near" ''
              export NEAR_ENV=local
              export NEAR_CLI_LOCALNET_RPC_SERVER_URL=http://localhost:33300
              # direnv sets PROJ_ROOT
              exec "${pkgs.nodePackages.near-cli}/bin/near" --keyPath $PROJ_ROOT/src/kuutamod/.data/near/localnet/owner/validator_key.json "$@"
            '')
          ]
          ++ self'.packages.kuutamod.buildInputs;
        CORE_CONTRACTS = self.inputs.core-contracts;
        inherit (self'.packages.kuutamod) nativeBuildInputs;
        NEARD_VERSION = "${self'.packages.neard.version}";
        passthru = {
          inherit formatters;
        };
      };
    };
}
