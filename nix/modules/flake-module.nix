{ self, ... }:
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
      kuutamod = ./kuutamod;
      default = self.nixosModules.kuutamod;
    };
  };
}
