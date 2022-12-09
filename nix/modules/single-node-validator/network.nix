{ config, lib, pkgs, ... }:
let
  cfg = config.kuutamo.network;
in
{
  imports = [ ../networkd.nix ];

  options = {
    # FIXME: support mac address here for matching interface
    kuutamo.network.interface = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = "eth0";
    };
    kuutamo.network.ipv4.address = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
    };

    kuutamo.network.ipv4.cidr = lib.mkOption {
      type = lib.types.str;
      default = "32";
    };

    kuutamo.network.ipv4.gateway = lib.mkOption {
      type = lib.types.str;
    };

    kuutamo.network.ipv6.address = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
    };

    kuutamo.network.ipv6.cidr = lib.mkOption {
      type = lib.types.str;
      default = "128";
    };

    kuutamo.network.ipv6.gateway = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
    };
  };

  config = {

    assertions = [{
      assertion = cfg.ipv4.address != null || cfg.ipv6.address != null;
      message = ''
        At least one ipv4 or ipv6 address must be configured
      '';
    }
      {
        assertion = cfg.ipv4.address != null -> cfg.ipv4.gateway != null;
        message = ''
          No ipv4 gateway configured
        '';
      }
      {
        assertion = cfg.ipv6.address != null -> cfg.ipv6.gateway != null;
        message = ''
          No ipv6 gateway configured
        '';
      }];

    # we just have one interface called 'eth0'
    networking.usePredictableInterfaceNames = false;

    systemd.network = {
      enable = true;
      networks."ethernet".extraConfig = ''
        [Match]
        Name = ${cfg.interface}

        [Network]
        ${lib.optionalString (cfg.ipv4.address != null) ''
          Address = ${cfg.ipv4.address}/${cfg.ipv4.cidr}
          Gateway = ${cfg.ipv4.gateway}
        ''}
        ${lib.optionalString (cfg.ipv6.address != null) ''
          Address = ${cfg.ipv6.address}/${cfg.ipv6.cidr}
          Gateway = ${cfg.ipv6.gateway}
        ''}
      '';
    };
  };
}
