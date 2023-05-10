{ config, lib, pkgs, ... }:

let
  cfg = config.kuutamo.deployConfig;
  settingsFormat = pkgs.formats.toml { };
in
{
  options.kuutamo.deployConfig = lib.mkOption {
    default = { };
    description = lib.mdDoc "toml configuration from kneard-mgr cli";
    inherit (settingsFormat) type;
  };

  # deployConfig is optional
  config = lib.mkIf (cfg != { }) {
    networking.hostName = cfg.name;

    kuutamo.disko.disks = cfg.disks;

    # FIXME: this should be provided by kneard-ctl
    users.extraUsers.root.openssh.authorizedKeys.keys = cfg.public_ssh_keys;

    kuutamo.kneard.publicAddress = cfg.ipv6_address or cfg.ipv4_address;

    kuutamo.network.macAddress = cfg.mac_address or null;
    kuutamo.network.interface = cfg.interface;

    kuutamo.network.ipv4.address = cfg.ipv4_address;
    kuutamo.network.ipv4.gateway = cfg.ipv4_gateway;
    kuutamo.network.ipv4.cidr = cfg.ipv4_cidr;

    kuutamo.network.ipv6.address = cfg.ipv6_address or null;
    kuutamo.network.ipv6.gateway = cfg.ipv6_gateway or null;
    kuutamo.network.ipv6.cidr = cfg.ipv6_cidr or 128;

    kuutamo.telegraf.hasMonitoring = cfg.telegraf_has_monitoring or false;
    kuutamo.telegraf.configHash = cfg.telegraf_config_hash or "";
    kuutamo.exporter.accountId = cfg.validator_account_id or null;
  };
}
