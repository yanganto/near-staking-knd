{ self, ... }:
{
  flake = { ... }: {
    nixosModules = {
      neard = ./neard;
      kuutamod = ./kuutamod;
      default = self.nixosModules.kuutamod;
    };
  };
}
