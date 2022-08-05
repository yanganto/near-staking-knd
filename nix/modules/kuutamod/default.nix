{ config
, lib
, pkgs
, ...
}:
let
  kuutamod = pkgs.callPackage ../../pkgs/kuutamod.nix { };
  cfg = config.kuutamo.kuutamod;
  cfgNeard = config.kuutamo.neard;
in
{
  options.kuutamo.kuutamod = {
    validatorKeyFile = lib.mkOption {
      type = lib.types.path;
      description = ''
        A file which contains a public and private key for local account which belongs to the only local network validator (validator_key.json of the validator).
      '';
    };
    validatorNodeKeyFile = lib.mkOption {
      type = lib.types.path;
      description = ''
        A file which contains a public and private key for the validator node (node_key.json of the validator)
      '';
    };
    nodeId = lib.mkOption {
      type = lib.types.str;
      default = config.networking.hostName;
      description = ''
        Node ID used in logs
      '';
    };
    consulTokenFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        File containing consul token file used for authenticating consul agent.
        See https://www.consul.io/docs/security/acl/acl-tokens
      '';
    };
    accountId = lib.mkOption {
      type = lib.types.str;
      default = "default";
      description = ''
        NEAR Account id of the validator. This ID will be used to acquire
        leadership in consul. It should be the same for all nodes that share the
        same validator key.
      '';
    };
    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = ''
        Whether to open ports used by neard
      '';
    };
  };

  imports = [
    ../neard
  ];

  config = {
    services.consul = {
      enable = true;
      extraConfig = {
        retry_interval = "1s";
        # high-perf: https://www.consul.io/docs/install/performance
        performance.raft_multiplier = 2;
        # Allow user to have an external consul server for consensus.
        server = lib.mkDefault true;
      };
    };

    # kuutamo.neard and kuutamod cannot be used at the same time
    kuutamo.neard.enable = false;

    environment.systemPackages = [
      # for kuutamoctl
      kuutamod
    ];

    # this is useful for kuutamodctl
    environment.variables."KUUTAMO_ACCOUNT_ID" = cfg.accountId;

    systemd.services.kuutamod = {
      wantedBy = [ "multi-user.target" ];
      # we want to restart the service ourself manually
      reloadIfChanged = true;

      inherit (config.systemd.services.neard) path;

      serviceConfig = config.systemd.services.neard.serviceConfig // {
        Environment = [
          "KUUTAMO_NEARD_HOME=/var/lib/neard"
          "KUUTAMO_NODE_ID=${cfg.nodeId}"
          "KUUTAMO_ACCOUNT_ID=${cfg.accountId}"
          "KUUTAMO_VOTER_NODE_KEY=/var/lib/neard/voter_node_key.json"
          "KUUTAMO_VALIDATOR_KEY=${cfg.validatorKeyFile}"
          "KUUTAMO_VALIDATOR_NODE_KEY=${cfg.validatorNodeKeyFile}"
        ] ++ lib.optional (cfg.consulTokenFile != null) "KUUTAMO_CONSUL_TOKEN_FILE=${cfg.consulTokenFile}";

        RuntimeDirectory = "kuutamod";

        ExecReload = "${pkgs.writeShellScript "kuutamod-schedule-reload" ''
          touch /run/kuutamod/restart
        ''}";

        # this script is run as root
        ExecStartPre =
          config.systemd.services.neard.serviceConfig.ExecStartPre
            ++ [
            "+${pkgs.writeShellScript "kuutamod-setup" ''
                set -eux -o pipefail
                # Generate voter node key
                if [[ ! -f /var/lib/neard/voter_node_key.json ]]; then
                  mv /var/lib/neard/node_key.json /var/lib/neard/voter_node_key.json
                fi
              ''}"
          ];
        ExecStart = "${kuutamod}/bin/kuutamod";
      };
    };

    networking.firewall.allowedTCPPorts = lib.optionals cfg.openFirewall [
      # standard neard network port, also used in validator mode
      24567
      # neard network port when run as voter mode
      24568
    ];
  };
}
