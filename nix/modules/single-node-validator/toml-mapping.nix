{ config, lib, pkgs, ... }:

let
  cfg = config.kuutamo.deployConfig;

  settingsFormat = pkgs.formats.toml { };
in
{
  options.kuutamo.deployConfig = lib.mkOption {
    default = { };
    description = lib.mdDoc "toml configuration from kuutamo cli";
    type = settingsFormat.type;
  };
  # deployConfig is optional
  config = lib.mkIf (cfg != { }) {
    networking.hostName = cfg.name;

    # FIXME: Do we want this for debugging?
    # users.extraUsers.root.hashedPassword = "$6$u9LHxoCmgitOlJq3$ra347e9QiAwntV2rm8gHBA23bJSZ8nrU6oJK6fU2Cnbz8Vh0xoWSCqFkx5WgUFJnPvwziTdusJ3lR2HjlV.bx0";

    # FIXME: this should be provided by kuutamoctl
    users.extraUsers.root.openssh.authorizedKeys.keys = cfg.public_ssh_keys;

    kuutamo.network.ipv4.address = cfg.ipv4_address;
    kuutamo.network.ipv4.gateway = cfg.ipv4_gateway;
    kuutamo.network.ipv4.cidr = cfg.ipv4_cidr;

    kuutamo.network.ipv6.address = cfg.ipv6_address;
    kuutamo.network.ipv6.gateway = cfg.ipv6_gateway;
    kuutamo.network.ipv6.cidr = cfg.ipv6_cidr;

    # FIXME: how to upload these?
    kuutamo.kuutamod.validatorKeyFile = "/var/lib/secrets/validator_key.json";
    kuutamo.kuutamod.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
  };
}
