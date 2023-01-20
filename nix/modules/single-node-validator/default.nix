{ ... }: {
  imports = [
    ../network.nix
    ../hardware.nix
    ../consul.nix
    ../toml-mapping.nix
  ];

  system.stateVersion = "22.05";

  # FIXME: how to upload these?
  kuutamo.kuutamod.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kuutamod.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
}
