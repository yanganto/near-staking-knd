{ config, pkgs, lib, ... }:
let
  cfg = config.kuutamo.exporter or { accountId = ""; externalRpc = ""; };
in
{
  options.kuutamo.exporter = {
    accountId = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = lib.mdDoc ''validator account id'';
    };
    externalRpc = lib.mkOption {
      type = lib.types.str;
      description = lib.mdDoc ''public rpc source'';
    };
    package = lib.mkOption {
      type = lib.types.package;
      defaultText = lib.literalExpression ".#near-prometheus-exporter";
      description = lib.mdDoc "The near prometheus exporter package to use in our service";
    };
  };
  config = lib.mkIf (cfg.accountId != null) {
    services.telegraf.extraConfig.inputs.prometheus.urls = [
      "http://[::1]:9333"
    ];

    systemd.services.near-prometheus-exporter = {
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Restart = "always";
        ExecStart = ''${cfg.package}/bin/near-exporter \
          -accountId ${cfg.accountId} \
          -addr [::1]:9333 \
          -external-rpc ${cfg.externalRpc} \
          -url http://localhost:3030
        '';
        RestartSec = 2;
        Type = "simple";

        DynamicUser = true;
        PrivateTmp = "yes";
        PrivateUsers = "yes";
        PrivateDevices = "yes";
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = "yes";
        ProtectClock = "yes";
        ProtectControlGroups = "yes";
        ProtectKernelLogs = "yes";
        ProtectKernelModules = "yes";
        ProtectKernelTunables = "yes";
        ProtectProc = "invisible";
        CapabilityBoundingSet = "CAP_NET_BIND_SERVICE";
      };
    };

    systemd.services.telegraf.serviceConfig = {
      ExecStartPre = [
        "+${pkgs.writeShellScript "pre-start" ''
          cat > /var/log/telegraf/account-id <<EOF
          near,account_id=${cfg.accountId} v=0i
          EOF
        ''}"
      ];
    };
  };
}
