{ lib, ... }: {
  imports = [
    ../network.nix
    ../hardware.nix
    ../consul.nix
    ../toml-mapping.nix
  ];

  # we want `kuutamo update` to also restart `kneard.service`
  systemd.services.kneard.reloadIfChanged = lib.mkForce false;
  system.stateVersion = "22.05";

  kuutamo.kneard.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kneard.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
}
