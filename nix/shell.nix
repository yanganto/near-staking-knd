{ self, ... }: {
  perSystem =
    { config
    , self'
    , inputs'
    , pkgs
    , ...
    }:
    let
      formatters = [
        # our meta-formatter
        pkgs.treefmt

        # nix
        pkgs.nixpkgs-fmt
        # rust
        pkgs.rustfmt
        pkgs.clippy
        # python
        pkgs.black
        pkgs.mypy
        pkgs.python3.pkgs.flake8
      ];
    in
    {
      devShells.default = pkgs.mkShell {
        buildInputs =
          formatters
          ++ [
            # tasks and automation
            pkgs.just
            pkgs.jq
            self'.packages.near-cli
            self'.packages.rocksdb-tools
            pkgs.nix-update

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
        AWS_PROFILE = "mfa";
        nativeBuildInputs = self'.packages.kuutamod.nativeBuildInputs;
        passthru = {
          inherit formatters;
        };
      };
    };
}
