{ self, ... }:
{
  flake = { ... }:
    let
      inherit (self.inputs.nixpkgs) lib;
    in
    {
      nixosConfigurations = {
        single-node-validator-mainnet = lib.nixosSystem {
          system = "x86_64-linux";
          modules = [
            self.nixosModules.single-node-validator-mainnet
          ];
        };

        single-node-validator-testnet = lib.nixosSystem {
          system = "x86_64-linux";
          modules = [
            self.nixosModules.single-node-validator-testnet
          ];
        };
      };
    };
}
