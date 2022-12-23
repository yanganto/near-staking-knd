{ self, inputs, ... }:
{
  flake = { ... }: {
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
      neard-mainnet = { pkgs, lib, ... }: {
        kuutamo.neard.package = lib.mkDefault self.packages.${pkgs.stdenv.hostPlatform.system}.neard;
        imports = [
          ./neard/mainnet
        ];
      };
      kuutamo-binary-cache = ./binary-cache;
      kuutamod = { pkgs, ... }: {
        imports = [
          ./kuutamod
        ];
        kuutamo.kuutamod.package = self.packages.${pkgs.stdenv.hostPlatform.system}.kuutamod;
      };

      disko-partitioning-script = ./disko-partitioning-script.nix;
      networkd = ./networkd.nix;

      single-node-validator = {
        imports = [
          self.nixosModules.kuutamod
          self.nixosModules.disko-partitioning-script
          self.nixosModules.networkd
          self.nixosModules.kuutamo-binary-cache
          inputs.srvos.nixosModules.common
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

      default = self.nixosModules.kuutamod;
    };
  };
}
