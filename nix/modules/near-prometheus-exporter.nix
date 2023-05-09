{ config, lib, ... }:
let
  cfg = config.kuutamo.exporter or { accountId = ""; externalRpc = ""; };
in
{
  options.kuutamo.exporter = {
    accountId = lib.mkOption {
      type = lib.types.str;
      description = ''validator account id'';
    };
    externalRpc = lib.mkOption {
      type = lib.types.str;
      description = ''public rpc source'';
    };
    package = lib.mkOption {
      type = lib.types.package;
      defaultText = lib.literalExpression "pkgs.near-prometheus-exporter";
      description = "The near prometheus exporter package to use in our service";
    };
  };
  config = {
    systemd.services.near-prometheus-exporter = {
      enable = if config.kuutamo.exporter.accountId == "" then false else true;
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
  };
}
