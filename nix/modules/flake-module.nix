{ self, inputs, ... }:
{
  flake = _: {
    nixosModules = {
      neard = { pkgs, lib, ... }: {
        kuutamo.neard.package = lib.mkDefault self.packages.${pkgs.system}.neard;
        imports = [
          ./neard
        ];
      };
      neard-testnet = { pkgs, lib, ... }: {
        kuutamo.neard.package = lib.mkDefault self.packages.${pkgs.system}.neard-unstable;
        imports = [
          ./neard/testnet
        ];
      };
      neard-mainnet = { pkgs, ... }: rec {
        kuutamo.neard.package = pkgs.callPackage ../pkgs/neard/patch-stable.nix
          {
            # FIXME when we build on more target
            inherit (inputs.fenix.outputs.packages.x86_64-linux) fromToolchainFile;
            neardPatches = kuutamo.neard.neardPatches or [ ];
            revisionNumber = kuutamo.neard.revisionNumber or null;
          };
        imports = [ ./neard/mainnet ];
      };
      telegraf = ./telegraf.nix;
      kuutamo-binary-cache = ./binary-cache;
      kneard = { pkgs, ... }: {
        imports = [
          ./kneard
        ];
        kuutamo.kneard.package = self.packages.${pkgs.stdenv.hostPlatform.system}.kneard;
      };

      disko-partitioning-script = ./disko-partitioning-script.nix;
      networkd = ./networkd.nix;
      near-prometheus-exporter = { pkgs, ... }: {
        imports = [
          ./near-prometheus-exporter.nix
        ];
        kuutamo.exporter.package = self.packages.${pkgs.stdenv.hostPlatform.system}.near-prometheus-exporter;
      };

      single-node-validator = {
        imports = [
          self.nixosModules.kneard
          self.nixosModules.disko-partitioning-script
          self.nixosModules.networkd
          self.nixosModules.near-prometheus-exporter
          self.nixosModules.kuutamo-binary-cache
          inputs.srvos.nixosModules.server
          inputs.disko.nixosModules.disko
        ];
      };

      single-node-validator-mainnet = {
        imports = [
          self.nixosModules.single-node-validator
          self.nixosModules.neard-mainnet
          ./single-node-validator/mainnet.nix
        ];
      };

      single-node-validator-testnet = {
        imports = [
          self.nixosModules.single-node-validator
          self.nixosModules.neard-testnet
          ./single-node-validator/testnet.nix
        ];
      };

      single-node-archiver = { pkgs, ... }: {
        imports = [
          self.nixosModules.disko-partitioning-script
          self.nixosModules.networkd
          self.nixosModules.near-prometheus-exporter
          self.nixosModules.kuutamo-binary-cache
          inputs.srvos.nixosModules.server
          inputs.disko.nixosModules.disko
        ];
        kuutamo.near-staking-analytics.package = self.packages.${pkgs.stdenv.hostPlatform.system}.near-staking-analytics;
        nixpkgs.config.allowUnfree = true;
      };

      single-node-archiver-mainnet = {
        imports = [
          self.nixosModules.single-node-archiver
          self.nixosModules.neard-mainnet
          ./single-node-archiver/mainnet.nix
        ];
      };

      single-node-archiver-testnet = {
        imports = [
          self.nixosModules.single-node-archiver
          self.nixosModules.neard-testnet
          ./single-node-archiver/testnet.nix
        ];
      };

      default = self.nixosModules.kneard;
    };
  };
}
