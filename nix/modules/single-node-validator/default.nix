{ lib, self, ... }: {
  imports = [
    ../network.nix
    ../hardware.nix
    ../consul.nix
    ../toml-mapping.nix
  ];

  environment.etc."system-info.toml".text = lib.mkDefault ''
    git_sha = "${self.rev or "dirty"}"
    git_commit_date = "${self.lastModifiedDate}"
  '';

  # we want `kuutamo update` to also restart `kuutamod.service`(for kneard)
  systemd.services.kuutamod.reloadIfChanged = lib.mkForce false;
  system.stateVersion = "22.05";

  kuutamo.kneard.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kneard.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
}
