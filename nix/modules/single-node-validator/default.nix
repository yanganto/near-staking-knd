{ config, lib, self, pkgs, ... }:
let
  inherit (self.packages.${pkgs.hostPlatform.system}) kneard;
in
{
  imports = [
    ../network.nix
    ../hardware.nix
    ../consul.nix
    ../near-prometheus-exporter.nix
    ../toml-mapping.nix
  ];

  environment.etc."system-info.toml".text = lib.mkDefault ''
    git_sha = "${self.rev or "dirty"}"
    git_commit_date = "${self.lastModifiedDate}"
  '';
  system.activationScripts.nixos-upgrade = ''
    ${config.systemd.package}/bin/systemd-run --collect --unit nixos-upgrade echo level=info message="kneard node updated" $(${kneard}/bin/kneard-ctl system-info --inline)
  '';

  # we want `kuutamo update` to also restart `kuutamod.service`(for kneard)
  systemd.services.kuutamod.reloadIfChanged = lib.mkForce false;
  system.stateVersion = "22.05";

  kuutamo.kneard.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kneard.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
}
