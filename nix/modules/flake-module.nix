{ self, ... }:
{
  flake = { ... }: {
    nixosModules = {
      neard = ./neard;
      neard-testnet = ./neard/testnet;
      neard-mainnet = ./neard/mainnet;
      neard-shardnet = { pkgs, ... }: {
        kuutamo.neard.package = self.packages.${pkgs.system}.neard-shardnet;
        imports = [
          ./neard/shardnet
        ];
      };
      kuutamod = ./kuutamod;
      default = self.nixosModules.kuutamod;
    };
  };
}
