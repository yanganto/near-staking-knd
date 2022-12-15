{ lib, pkgs, config, ... }: {
  imports = [
    ./network.nix
    ./hardware.nix
    ./consul.nix
  ];


  # FIXME this should be configurable
  networking.hostName = "nixos";

  # FIXME: this is only for debugging and we won't keep this for later
  users.extraUsers.root.hashedPassword = "$6$u9LHxoCmgitOlJq3$ra347e9QiAwntV2rm8gHBA23bJSZ8nrU6oJK6fU2Cnbz8Vh0xoWSCqFkx5WgUFJnPvwziTdusJ3lR2HjlV.bx0";

  # FIXME: this should be provided by kuutamoctl
  users.extraUsers.root.openssh.authorizedKeys.keys = [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKbBp2dH2X3dcU1zh+xW3ZsdYROKpJd3n13ssOP092qE joerg@turingmachine"
  ];

  # FIXME: this should be provided by kuutamoctl
  kuutamo.network.ipv4.address = "199.127.63.197";
  kuutamo.network.ipv4.gateway = "199.127.63.1";
  kuutamo.network.ipv4.cidr = "24";
  kuutamo.network.ipv6.address = "2605:9880:400:700:8:b10c:1932:3224";
  kuutamo.network.ipv6.gateway = "2605:9880:400::1";
  kuutamo.network.ipv6.cidr = "48";

  # FIXME: how to upload these?
  kuutamo.kuutamod.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kuutamod.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";

  system.stateVersion = "22.05";
}
