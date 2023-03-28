{ config
, lib
, pkgs
, ...
}:
let
  cfg = config.kuutamo.kneard;
in
{
  options.kuutamo.kneard = {
    validatorKeyFile = lib.mkOption {
      type = lib.types.either lib.types.path lib.types.str;
      description = ''
        A file which contains a public and private key for local account which belongs to the only local network validator (validator_key.json of the validator).
      '';
    };
    validatorNodeKeyFile = lib.mkOption {
      type = lib.types.either lib.types.path lib.types.str;
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

    package = lib.mkOption {
      type = lib.types.package;
      description = lib.mdDoc ''
        Kuutamod package to use
      '';
    };

    publicAddress = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The ip addresses of the validator is *directly* reachable.
        Kuutamod will add the configured validator node key and port number of this node to these addresses and expects
        each entry to be an ip address without the public key part
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

    # We observed once that in early start of the executable a SIGHUP signal
    # stopped consul presumably because the signal handler was not yet
    # installed. This is a workaround for that rare case.
    systemd.services.consul.serviceConfig.Restart = lib.mkForce "always";

    # kuutamo.neard and kneard cannot be used at the same time
    kuutamo.neard.enable = false;

    environment.systemPackages = [
      # for kneard-ctl
      cfg.package
    ];

    # this is useful for kneardctl
    environment.variables =
      {
        "KUUTAMO_ACCOUNT_ID" = cfg.accountId;
      }
      // lib.optionalAttrs (cfg.consulTokenFile != null) {
        "KUUTAMO_CONSUL_TOKEN_FILE" = "/run/kneard/consul-token";
      };

    # If failover / kneard fails for what ever reason, this service allows to
    # start neard manually.
    # Do NOT start this service if any kneard for this validator key is
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

    # We still use kuutamod here for backward compatible
    systemd.services.kuutamod = {
      wantedBy = [ "multi-user.target" ];
      # we want to restart the service ourself manually
      reloadIfChanged = true;

      inherit (config.systemd.services.neard) path;

      serviceConfig =
        config.systemd.services.neard.serviceConfig
        // {
          LoadCredential = lib.mkDefault [
            "validator-key-file:${cfg.validatorKeyFile}"
            "validator-node-key-file:${cfg.validatorNodeKeyFile}"
          ];

          Environment =
            [
              "KUUTAMO_NEARD_HOME=/var/lib/neard"
              "KUUTAMO_NODE_ID=${cfg.nodeId}"
              "KUUTAMO_ACCOUNT_ID=${cfg.accountId}"
              "KUUTAMO_VOTER_NODE_KEY=/var/lib/neard/voter_node_key.json"
              "KUUTAMO_VALIDATOR_KEY=%d/validator-key-file"
              "KUUTAMO_VALIDATOR_NODE_KEY=%d/validator-node-key-file"
            ]
            ++ lib.optional (cfg.consulTokenFile != null) "KUUTAMO_CONSUL_TOKEN_FILE=${cfg.consulTokenFile}"
            ++ lib.optional (cfg.publicAddress != null) "KUUTAMO_PUBLIC_ADDRESS=${cfg.publicAddress}";

          RuntimeDirectory = "kneard";

          ExecReload = [
            "+${pkgs.writeShellScript "kneard-schedule-reload" ''
              set -x
              touch /run/kneard/restart

              ${lib.optionalString (cfg.consulTokenFile != null) ''
                # We need those keys for kneard-ctl as root
                # We copy the token from the service here to make things like systemd's LoadCredential and secrets from vault work.
                install -m400 "$KUUTAMO_CONSUL_TOKEN_FILE" /run/kneard/consul-token
              ''}

              # reload consul token file
              kill -SIGUSR1 $MAINPID
            ''}"
          ];

          # If neard goes out-of-memory, we want to keep kneard running.
          OOMPolicy = "continue";

          # in addition to ipv4/ipv6 we also need unix sockets
          RestrictAddressFamilies = [ "AF_UNIX" "AF_INET" "AF_INET6" ];

          # this script is run as root
          ExecStartPre =
            lib.optional (cfg.consulTokenFile != null) "+${pkgs.writeShellScript "kneard-consul-token" ''
              set -eux -o pipefail
              install -m400 "$KUUTAMO_CONSUL_TOKEN_FILE" /run/kneard/consul-token
            ''}"
            ++ config.systemd.services.neard.serviceConfig.ExecStartPre
            ++ [
              "+${pkgs.writeShellScript "kneard-setup" ''
                set -eux -o pipefail
                # Generate voter node key
                if [[ ! -f /var/lib/neard/voter_node_key.json ]]; then
                  mv /var/lib/neard/node_key.json /var/lib/neard/voter_node_key.json
                fi
              ''}"
              # we need to execute this as the neard user so we get access to private tmp
            ];
          ExecStart = "${cfg.package}/bin/kneard";
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
