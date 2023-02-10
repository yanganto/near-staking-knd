{ lib, ... }: {
  imports = [
    ../network.nix
    ../hardware.nix
    ../consul.nix
    ../toml-mapping.nix
  ];

  # we want `kuutamo update` to also restart `kuutamod.service`
  systemd.services.kuutamod.reloadIfChanged = lib.mkForce false;
  system.stateVersion = "22.05";

  kuutamo.kuutamod.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kuutamod.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
}
