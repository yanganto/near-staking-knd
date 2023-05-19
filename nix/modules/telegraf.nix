{ config, pkgs, lib, ... }:
{
  options = {
    kuutamo.telegraf.configHash = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "telegraf config hash";
    };

    kuutamo.telegraf.hasMonitoring = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "has monitoring setting or not";
    };

    kuutamo.telegraf.hostname = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "the hostname tag on metrics";
    };
  };
  config = {
    services.telegraf = {
      enable = true;
      environmentFiles = lib.optionals config.kuutamo.telegraf.hasMonitoring [
        "/var/lib/secrets/telegraf"
        # this triggers a restart of telegraf when the config changes
        (pkgs.writeText "monitoring-configHash" config.kuutamo.telegraf.configHash)
      ];
      extraConfig = {
        agent.interval = "60s";
        agent.round_interval = true;
        agent.metric_batch_size = 10000;
        agent.collection_offset = "5s";
        agent.flush_interval = "60s";
        agent.flush_jitter = "40s";
        inputs = {
          cpu = {
            tags = {
              host = config.kuutamo.telegraf.hostname;
            };
          };
          prometheus.urls = [
            "http://localhost:3030/metrics"
            "http://localhost:2233/metrics"
          ];
          prometheus.tags = {
            host = config.kuutamo.telegraf.hostname;
          };
        };
        outputs = {
          http = lib.mkIf config.kuutamo.telegraf.hasMonitoring {
            url = "$MONITORING_URL";
            data_format = "prometheusremotewrite";
            username = "$MONITORING_USERNAME";
            password = "$MONITORING_PASSWORD";
          };
        };
      };
    };
  };
}
