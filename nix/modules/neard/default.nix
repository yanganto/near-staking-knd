{ config, lib, pkgs, ... }:
let
  cfg = config.kuutamo.neard;
in
{
  options.kuutamo.neard = {
    enable = lib.mkOption {
      type = lib.types.bool;
      description = "Whether to enable Neard";
      default = true;
    };
    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ../../pkgs/neard/stable.nix { };
      defaultText = lib.literalExpression "pkgs.neard";
      description = "The neard package to use in our service";
    };
    generateNodeKey = lib.mkOption {
      type = lib.types.bool;
      description = "Whether to generate Node key on boot";
      default = true;
    };

    s3DataBackupUrl = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "s3://near-protocol-public/backups/mainnet/rpc/latest";
      description = ''
        S3 backup url to load initial near database data from
      '';
    };

    configFile = lib.mkOption {
      type = lib.types.path;
      default = ./mainnet/config.json;
      description = "Configuration for the node in json format";
    };

    genesisFile = lib.mkOption {
      type = lib.types.path;
      default = ./mainnet/genesis.json;
      description = ''
        A file with all the data the network started with at genesis. This contains initial accounts, contracts, access keys, and other records which represents the initial state of the blockchain.
      '';
    };
  };

  imports = [
    ../grafana-agent
  ];

  config = {
    kuutamo.grafana-agent.scrapeTargets.neard.port = 3030;

    users.users.neard = {
      group = "neard";
      isSystemUser = true;
    };
    users.groups.neard = { };

    systemd.services.neard = {
      enable = config.kuutamo.neard.enable;
      wantedBy = [ "multi-user.target" ];
      path = [
        pkgs.awscli2
        cfg.package
      ];

      serviceConfig = {
        StateDirectory = "neard";
        TimeoutStartSec = "30min"; # downloading chain data can take some time...
        # this script is run as root
        ExecStartPre = [
          "+${pkgs.writeShellScript "neard-setup" ''
          set -eux -o pipefail
          # Boostrap chain data for new nodes
          if [[ ! -f /var/lib/neard/.finished ]]; then
            ${lib.optionalString (cfg.s3DataBackupUrl != null) ''
              aws s3 sync --delete ${cfg.s3DataBackupUrl} /var/lib/neard/data
              chown -R neard:neard /var/lib/neard/data
            ''}
            ${lib.optionalString (cfg.generateNodeKey) ''
              ${cfg.package}/bin/neard --home /var/lib/neard init
              chown neard:neard /var/lib/neard/node_key.json /var/lib/neard/validator_key.json
            ''}
            touch /var/lib/neard/.finished
          fi

          # Update configuration
          install -D -m755 -o neard -g neard ${cfg.configFile} /var/lib/neard/config.json
          ln -sf ${cfg.genesisFile} /var/lib/neard/genesis.json
        ''}"
        ];
        ExecStart = "${cfg.package}/bin/neard --home /var/lib/neard run";
        Restart = "always";

        User = "neard";
        Group = "neard";

        # New file permissions
        UMask = "0027"; # 0640 / 0750

        # Hardening measures
        # Sandboxing (sorted by occurrence in https://www.freedesktop.org/software/systemd/man/systemd.exec.html)

        ProtectSystem = "full";
        Type = "simple";
        ProtectHome = true;
        ProtectHostname = true;
        ProtectClock = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;

        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        PrivateMounts = true;
        # We do we have to disable this? Seems to be related to the wasm runtime
        # neard[1711]: thread '<unnamed>' panicked at 'unable to make memory readonly and executable: SystemCall(Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" })', /build/neard-1.25.0-vendor.tar.gz/wasmer-engine-universal-near/src/code_memory.rs:153:10
        #MemoryDenyWriteExecute = true;
        RemoveIPC = true;

        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" ];
        RestrictRealtime = true;
        RestrictSUIDSGID = true;

        LockPersonality = true;

        # Proc filesystem
        ProcSubset = "pid";
        ProtectProc = "invisible";

        RestrictNamespaces = true;

        SystemCallArchitectures = "native";
        # blacklist some syscalls
        SystemCallFilter = [ "~@cpu-emulation @debug @keyring @mount @obsolete @privileged @setuid @ipc" ];
      };
    };
  };
}
