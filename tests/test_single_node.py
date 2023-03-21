#!/usr/bin/env python3

import json
import signal
import time
from pathlib import Path
from tempfile import _TemporaryFileWrapper

from command import Command
from consul import Consul
from network import wait_for_port
from ports import Ports
from prometheus import query_prometheus_endpoint
from setup_localnet import NearNetwork


def assert_key_equal(expected: Path, current: Path) -> None:
    expected_key = json.loads(expected.read_text())
    current_key = json.loads(current.read_text())
    assert expected_key == current_key


def test_single_node(
    kneard: Path,
    command: Command,
    consul_with_acls: Consul,
    near_network: NearNetwork,
    ports: Ports,
    temporary_file: "_TemporaryFileWrapper[str]",
) -> None:
    # FIXME Just now we use the validator key of genesis node3 for our setup
    validator_key = near_network.home / "node3" / "validator_key.json"
    validator_node_key = near_network.home / "node3" / "node_key.json"

    neard_home = near_network.home / "kneard0"
    voter_node_key = neard_home / "voter_node_key.json"
    exporter_port = ports.allocate(3)
    validator_port = exporter_port + 1
    voter_port = exporter_port + 2

    consul_token = consul_with_acls.management_token
    assert consul_token is not None
    temporary_file.write(consul_token)
    # Check if kneard handles trailing newline gracefully
    temporary_file.write("\n")
    temporary_file.flush()

    env = dict(
        KUUTAMO_CONSUL_URL=f"http://127.0.0.1:{consul_with_acls.http_port}",
        KUUTAMO_EXPORTER_ADDRESS=f"127.0.0.1:{exporter_port}",
        KUUTAMO_VALIDATOR_NETWORK_ADDR=f"127.0.0.1:{validator_port}",
        KUUTAMO_VOTER_NETWORK_ADDR=f"127.0.0.1:{voter_port}",
        KUUTAMO_VALIDATOR_KEY=str(validator_key),
        KUUTAMO_PUBLIC_ADDRESS="127.0.0.1",
        KUUTAMO_VALIDATOR_NODE_KEY=str(validator_node_key),
        KUUTAMO_VOTER_NODE_KEY=str(voter_node_key),
        KUUTAMO_NEARD_HOME=str(neard_home),
        KUUTAMO_NEARD_BOOTNODES=near_network.bootstrap_node,
        KUUTAMO_CONSUL_TOKEN_FILE=temporary_file.name,
        RUST_BACKTRACE="1",
    )

    proc = command.run([str(kneard)], extra_env=env)
    try:
        wait_for_port("127.0.0.1", exporter_port, proc=proc)

        # kneard reduces its oom score a bit
        assert Path(f"/proc/{proc.pid}/oom_score_adj").read_text() == "100\n"

        # Should start on voter port (This check might racy)
        wait_for_port("127.0.0.1", voter_port, proc=proc)
        while True:
            res = query_prometheus_endpoint("127.0.0.1", exporter_port)
            if res.get('kneard_state{type="Validating"}') == "1":
                break
            time.sleep(0.1)

        # Should use validator port.
        wait_for_port("127.0.0.1", validator_port)
        assert_key_equal(validator_node_key, neard_home / "node_key.json")
        assert_key_equal(validator_key, neard_home / "validator_key.json")
        time.sleep(5)  # it should stay master at this point
        res = query_prometheus_endpoint("127.0.0.1", exporter_port)
        # only one needed restart to get into validator state
        assert res.get("kneard_neard_restarts") == "1"
        assert int(res.get("kneard_uptime", "0")) > 0
        assert res.get('kneard_state{type="Validating"}') == "1"
        assert res.get('kneard_state{type="Registering"}') == "0"
        assert res.get('kneard_state{type="Shutdown"}') == "0"
        assert res.get('kneard_state{type="Startup"}') == "0"
        assert res.get('kneard_state{type="Syncing"}') == "0"
        assert res.get('kneard_state{type="Voting"}') == "0"
    finally:
        # terminate kneard -> neard properly so we can cleanup the temporary directory
        proc.send_signal(signal.SIGTERM)
        proc.wait()
