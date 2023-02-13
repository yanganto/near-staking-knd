{ config
, lib
, ...
}:
let
  cfg = config.kuutamo.near-staking-analytics;
in
{
  options.kuutamo.near-staking-analytics = {
    enable = lib.mkEnableOption "near analytics backend";

    mongodb = lib.mkOption {
      type = lib.types.str;
      default = "mongodb://127.0.0.1:27017/near-analytics";
    };

    package = lib.mkOption {
      type = lib.types.package;
      defaultText = lib.literalExpression "pkgs.near-staking-analytics";
      description = "The near analytics package to use in our service";
    };

    port = lib.mkOption {
      type = lib.types.int;
      default = 8080;
    };

    domain = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.near-staking-analytics = {
      wantedBy = [ "multi-user.target" ];
      environment = {
        MONGO = cfg.mongodb;
        PORT = toString cfg.port;
        TESTNET_POSTGRESQL_CONNECTION_STRING = "postgresql://public_readonly:nearprotocol@testnet.db.explorer.indexer.near.dev/testnet_explorer";
        TESTNET_NEAR_RPC_URL = "https://rpc.testnet.near.org";
        TESTNET_NEAR_ARCHIVAL_RPC_URL = "https://archival-rpc.testnet.near.org";

        MAINNET_POSTGRESQL_CONNECTION_STRING = "postgresql://public_readonly:nearprotocol@mainnet.db.explorer.indexer.near.dev/mainnet_explorer";
        MAINNET_NEAR_RPC_URL = "https://rpc.mainnet.near.org";
        MAINNET_NEAR_ARCHIVAL_RPC_URL = "https://archival-rpc.mainnet.near.org";
      };
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/near-staking-analytics";
        DynamicUser = true;
      };
    };
    services.nginx = lib.mkIf (cfg.domain != null) {
      enable = true;
      virtualHosts.${cfg.domain} = {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:${toString cfg.port}";
          recommendedProxySettings = true;
        };
      };
    };
    networking.firewall.allowedTCPPorts = lib.mkIf (cfg.domain != null) [ 80 443 ];
  };
}
