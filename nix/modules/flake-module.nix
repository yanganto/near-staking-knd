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
      kuutamo-binary-cache = ./binary-cache;
      kuutamod = ./kuutamod;
      default = self.nixosModules.kuutamod;
    };
  };
}
