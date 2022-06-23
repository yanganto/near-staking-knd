{ makeTest'
}:
makeTest'
{
  name = "neard";
  nodes.server = { lib, ... }: {
    imports = [
      ../neard
    ];

    # FIXME no s3 available in nixos tests yet.
    kuutamo.neard.s3DataBackupUrl = null;
  };

  testScript = ''
    start_all()
    server.wait_for_unit("neard.service")
    # neard prometheus endpoint
    server.wait_until_succeeds("curl --silent http://127.0.0.1:3030/metrics")
    # near-prometheus-exporter endpoint
    server.wait_until_succeeds("curl --silent http://[::1]:9333/metrics")
    # check that node_key and validator key are present
    server.succeed("[[ -f /var/lib/neard/validator_key.json ]]")
    server.succeed("[[ -f /var/lib/neard/node_key.json ]]")
  '';
}
