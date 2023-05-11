(import ./lib.nix) ({ self, ... }: {
  name = "single-node-monitoring";
  nodes.server = { ... }: {
    imports = [
      self.nixosModules.kneard
      self.nixosModules.neard-mainnet
      self.nixosModules.telegraf
      self.nixosModules.near-prometheus-exporter
    ];

    services.consul.interface.bind = "eth0";
    # our consul wants an ipv6 address
    networking.interfaces.eth0 = {
      ipv6.addresses = [
        {
          address = "2001:1470:fffd:2097::";
          prefixLength = 64;
        }
      ];
    };
    services.consul.extraConfig.bootstrap_expect = 1;
    virtualisation.memorySize = 1024;

    kuutamo.kneard.validatorKeyFile = ./validator_key.json;
    kuutamo.kneard.validatorNodeKeyFile = ./node_key.json;

    kuutamo.telegraf = {
      configHash = "";
      hasMonitoring = false;
    };
    kuutamo.exporter = {
      accountId = "kuutamod0";
      externalRpc = "http://localhost:3030";
    };
  };

  testScript = ''
    start_all()
    server.wait_for_unit("kuutamod.service")
    server.wait_for_unit("near-prometheus-exporter.service")
    server.wait_for_unit("telegraf.service")

    # exporter prometheus endpoint
    server.wait_until_succeeds("curl --silent http://[::1]:9333/metrics")

    # telegraf prometheus endpoint
    server.wait_until_succeeds("curl --silent http://127.0.0.1:9273/metrics")
  '';
})
