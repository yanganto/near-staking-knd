{ near-staking-knd, ... }: {
  nixosConfigurations."validator-00" = near-staking-knd.inputs.nixpkgs.lib.nixosSystem {
    system = "x86_64-linux";
    modules = [
      near-staking-knd.nixosModules."single-node-validator-mainnet"
      near-staking-knd.nixosModules."qemu-test-profile"
      { kuutamo.deployConfig = builtins.fromTOML (builtins.readFile (builtins.path { name = "validator.toml"; path = ./validator-00.toml; })); }
    ];
  };
}
