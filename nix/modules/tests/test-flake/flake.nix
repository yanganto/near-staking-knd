{
  inputs.near-staking-knd.url = "/root/near-staking-knd";

  outputs = inputs: import ./configurations.nix inputs;
}
