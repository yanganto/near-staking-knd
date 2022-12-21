{ self, ... }:
{
  flake = { ... }:
    let
      inherit (self.inputs.nixpkgs) lib;
    in
    {
      nixosConfigurations =
        let
          dummyConfig = {
            networking.hostName = "nixos";

            kuutamo.network.ipv4.address = "199.127.63.197";
            kuutamo.network.ipv4.gateway = "199.127.63.1";
            kuutamo.network.ipv4.cidr = 24;
            kuutamo.network.ipv6.address = "2605:9880:400:700:8:b10c:1932:3224";
            kuutamo.network.ipv6.gateway = "2605:9880:400::1";
            kuutamo.network.ipv6.cidr = 48;

            users.extraUsers.root.openssh.authorizedKeys.keys = [
              "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKbBp2dH2X3dcU1zh+xW3ZsdYROKpJd3n13ssOP092qE joerg@turingmachine"
            ];
          };
          validator = {
            kuutamo.kuutamod.validatorKeyFile = "/var/lib/secrets/validator_key.json";
            kuutamo.kuutamod.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
          };
        in
        {
          single-node-validator-mainnet = lib.nixosSystem {
            system = "x86_64-linux";
            modules = [
              dummyConfig
              validator
              self.nixosModules.single-node-validator-mainnet
            ];
          };
          single-node-standby-mainnet = lib.nixosSystem {
            system = "x86_64-linux";
            modules = [
              dummyConfig
              self.nixosModules.single-node-validator-mainnet
            ];
          };

          single-node-validator-testnet = lib.nixosSystem {
            system = "x86_64-linux";
            modules = [
              dummyConfig
              validator
              self.nixosModules.single-node-validator-testnet
            ];
          };
        };
    };
}
