{ self, ... }:
{
  perSystem = { ... }: {
    nixosModules = {
      neard = ./neard;
      kuutamod = ./kuutamod;
      default = self.nixosModules.kuutamod;
    };
  };
}
