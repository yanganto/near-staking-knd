{ near-staking-knd, ... }: {
  nixosConfigurations."validator-00" = near-staking-knd.inputs.nixpkgs.lib.nixosSystem {
    system = "x86_64-linux";
    modules = [
      near-staking-knd.nixosModules."single-node-validator-mainnet"
      { kuutamo.deployConfig = builtins.fromTOML (builtins.readFile ./validator-00.toml); }
    ];
  };
}
