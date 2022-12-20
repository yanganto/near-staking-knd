{ config
, lib
, pkgs
, ...
}:
let
  kuutamod = pkgs.callPackage ../../pkgs/kuutamod.nix { };
  cfg = config.kuutamo.kuutamod;
in
{
  options.kuutamo.kuutamod = {
    validatorKeyFile = lib.mkOption {
      type = (lib.types.either lib.types.path lib.types.str);
      description = ''
        A file which contains a public and private key for local account which belongs to the only local network validator (validator_key.json of the validator).
      '';
    };
    validatorNodeKeyFile = lib.mkOption {
      type = (lib.types.either lib.types.path lib.types.str);
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
      type = lib.types.nullOr (lib.types.either lib.types.path lib.types.str);
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
    publicAddresses = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = ''
        Comma-separated list of ip addresses to be written to neard configuration on which the validator is *directly* reachable.
        Kuutamod will add the configured validator node key and port number of this node to these addresses.
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
    environment.variables =
      {
        "KUUTAMO_ACCOUNT_ID" = cfg.accountId;
      }
      // lib.optionalAttrs (cfg.consulTokenFile != null) {
        "KUUTAMO_CONSUL_TOKEN_FILE" = "/run/kuutamod/consul-token";
      };

    # If failover / kuutamod fails for what ever reason, this service allows to
    # start neard manually.
    # Do NOT start this service if any kuutamod for this validator key is
    # signing or else it will result in double-signing
    # This service assumes that configuration is already mostly inplace i.e. a
    # neard backup and neard's config.json
    systemd.services.neard-manual = {
      inherit (config.systemd.services.neard) path;

      serviceConfig =
        config.systemd.services.neard.serviceConfig
        // {
          Environment = [
            "VALIDATOR_KEY=${cfg.validatorKeyFile}"
            "VALIDATOR_NODE_KEY=${cfg.validatorNodeKeyFile}"
          ];
          ExecStartPre = [
            "+${pkgs.writeShellScript "neard-key-setup-setup" ''
              install -m400 -o neard -g neard "$VALIDATOR_KEY" /var/lib/neard/validator_key.json
              install -m400 -o neard -g neard "$VALIDATOR_NODE_KEY" /var/lib/neard/node_key.json
            ''}"
          ];
        };
    };

    systemd.services.kuutamod = {
      wantedBy = [ "multi-user.target" ];
      # we want to restart the service ourself manually
      reloadIfChanged = true;

      inherit (config.systemd.services.neard) path;

      serviceConfig =
        config.systemd.services.neard.serviceConfig
        // {
          Environment =
            [
              "KUUTAMO_NEARD_HOME=/var/lib/neard"
              "KUUTAMO_NODE_ID=${cfg.nodeId}"
              "KUUTAMO_ACCOUNT_ID=${cfg.accountId}"
              "KUUTAMO_VOTER_NODE_KEY=/var/lib/neard/voter_node_key.json"
              "KUUTAMO_VALIDATOR_KEY=${cfg.validatorKeyFile}"
              "KUUTAMO_VALIDATOR_NODE_KEY=${cfg.validatorNodeKeyFile}"
            ]
            ++ lib.optional (cfg.consulTokenFile != null) "KUUTAMO_CONSUL_TOKEN_FILE=${cfg.consulTokenFile}"
            ++ lib.optional (cfg.publicAddresses != [ ]) "KUUTAMO_PUBLIC_ADDRESSES=${lib.concatStringsSep "," cfg.publicAddresses}";

          RuntimeDirectory = "kuutamod";

          ExecReload = [
            "+${pkgs.writeShellScript "kuutamod-schedule-reload" ''
              set -x
              touch /run/kuutamod/restart

              ${lib.optionalString (cfg.consulTokenFile != null) ''
                # We need those keys for kuutamoctl as root
                # We copy the token from the service here to make things like systemd's LoadCredential and secrets from vault work.
                install -m400 "$KUUTAMO_CONSUL_TOKEN_FILE" /run/kuutamod/consul-token
              ''}

              # reload consul token file
              kill -SIGUSR1 $MAINPID
            ''}"
          ];

          # If neard goes out-of-memory, we want to keep kuutamod running.
          OOMPolicy = "continue";

          # in addition to ipv4/ipv6 we also need unix sockets
          RestrictAddressFamilies = [ "AF_UNIX" "AF_INET" "AF_INET6" ];

          # this script is run as root
          ExecStartPre =
            lib.optional (cfg.consulTokenFile != null) "+${pkgs.writeShellScript "kuutamod-consul-token" ''
              set -eux -o pipefail
              install -m400 "$KUUTAMO_CONSUL_TOKEN_FILE" /run/kuutamod/consul-token
            ''}"
            ++ config.systemd.services.neard.serviceConfig.ExecStartPre
            ++ [
              "+${pkgs.writeShellScript "kuutamod-setup" ''
                set -eux -o pipefail
                # Generate voter node key
                if [[ ! -f /var/lib/neard/voter_node_key.json ]]; then
                  mv /var/lib/neard/node_key.json /var/lib/neard/voter_node_key.json
                fi
              ''}"
              # we need to execute this as the neard user so we get access to private tmp
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
