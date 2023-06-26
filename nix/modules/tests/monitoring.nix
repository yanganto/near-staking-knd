(import ./lib.nix) ({ self, lib, pkgs, ... }: {
  name = "single-node-monitoring";
  nodes.server = { ... }: {
    imports = [
      self.nixosModules.kneard
      self.nixosModules.neard
      self.nixosModules.telegraf
      self.nixosModules.near-prometheus-exporter
    ];
    kuutamo.neard.chainId = "localnet";
    kuutamo.neard.configFile = null;
    kuutamo.neard.genesisFile = null;

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
    virtualisation.memorySize = 4096;

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

  testScript = { nodes, ... }: ''
    start_all()
    server.wait_for_unit("kuutamod.service")
    server.wait_for_unit("near-prometheus-exporter.service")
    server.wait_for_unit("telegraf.service")

    # exporter prometheus endpoint
    server.wait_until_succeeds("curl -v http://[::1]:9333/metrics >&2")

    # telegraf prometheus endpoint
    server.wait_until_succeeds("curl -v http://127.0.0.1:9273/metrics >&2")

    # neard endpoint
    server.wait_until_succeeds("curl -v http://0.0.0.0:3030/metrics >&2")

    out = server.succeed("${lib.getExe pkgs.telegraf} --test --config ${(pkgs.formats.toml {}).generate "config.toml" nodes.server.services.telegraf.extraConfig}")
    print(out)
    assert "near,account_id=" in out, "telegraf config should contain account_id"
  '';
})
