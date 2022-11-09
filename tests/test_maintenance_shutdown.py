#!/usr/bin/env python3

import os
import json
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from subprocess import Popen
from typing import Optional
import http.client

from command import Command
from consul import Consul
from network import wait_for_port
from ports import Ports
from prometheus import query_prometheus_endpoint
from setup_localnet import NearNetwork
import pytest
from typing import Any, List


def work_with_neard_versions(
    versions: List[str],
) -> Any:
    return pytest.mark.skipif(
        os.environ.get("NEARD_VERSION") not in versions,
        reason=f"Not suitable neard for current test, this test only for {versions}",
    )


def note(s: str) -> None:
    """Add note in log help to know the scenario"""
    print("\033[1;36m" + "#" * (len(s) + 6) + " \033[0m")
    print("\033[1;36mNOTE: " + s + " \033[0m")
    print("\033[1;36m" + "#" * (len(s) + 6) + " \033[0m")


@dataclass
class Kuutamod:
    proc: Popen
    exporter_port: int
    validator_port: int
    voter_port: int
    node_id: str
    control_socket: Path
    neard_home: Path


def query_neard_pid(host: str, port: int) -> Optional[int]:
    """Query neard pid with 3 times retry"""
    for i in range(3):
        try:
            conn = http.client.HTTPConnection(host, port)
            conn.request("GET", "/neard-pid")
            response = conn.getresponse()
            body = response.read().decode("utf-8")
            return int(body)
        except ConnectionRefusedError:
            if i == 2:
                raise ConnectionRefusedError
            pass
        time.sleep(i)
    return None


def run_node(
    neard_home: Path,
    consul: Consul,
    kuutamod: Path,
    command: Command,
    ports: Ports,
    near_network: NearNetwork,
) -> Kuutamod:
    exporter_port = ports.allocate(3)
    validator_port = exporter_port + 1
    voter_port = exporter_port + 2
    validator_key = near_network.home / "node3" / "validator_key.json"
    validator_node_key = near_network.home / "node3" / "node_key.json"
    voter_node_key = neard_home / "voter_node_key.json"
    node_id = str(neard_home.name)
    env = dict(
        KUUTAMO_CONSUL_URL=f"http://127.0.0.1:{consul.http_port}",
        KUUTAMO_NODE_ID=str(neard_home.name),
        KUUTAMO_EXPORTER_ADDRESS=f"127.0.0.1:{exporter_port}",
        KUUTAMO_VALIDATOR_NETWORK_ADDR=f"127.0.0.1:{validator_port}",
        KUUTAMO_VOTER_NETWORK_ADDR=f"127.0.0.1:{voter_port}",
        KUUTAMO_VALIDATOR_KEY=str(validator_key),
        KUUTAMO_VALIDATOR_NODE_KEY=str(validator_node_key),
        KUUTAMO_VOTER_NODE_KEY=str(voter_node_key),
        KUUTAMO_NEARD_HOME=str(neard_home),
        KUUTAMO_NEARD_BOOTNODES=near_network.boostrap_node,
        RUST_BACKTRACE="1",
    )
    proc = command.run([str(kuutamod)], extra_env=env)
    wait_for_port("127.0.0.1", exporter_port)

    return Kuutamod(
        proc=proc,
        exporter_port=exporter_port,
        node_id=node_id,
        validator_port=validator_port,
        voter_port=voter_port,
        control_socket=neard_home / "kuutamod.ctl",
        neard_home=neard_home,
    )


def send_maintenance_shutdown_comand(
    command: Command, kuutamoctl: Path, kuutamo_node: Kuutamod
) -> None:
    """Send maintenance shutdown with 3 times retry"""
    for i in range(3):
        proc = command.run(
            [
                str(kuutamoctl),
                "--control-socket",
                str(kuutamo_node.control_socket),
                "maintenance-shutdown",
                "1",  # Use one block window for maintenance shutdown in test
            ],
            stdout=subprocess.PIPE,
        )
        proc.wait()
        if proc.returncode == 0:
            return
        time.sleep(i)
    assert False


def check_blocking(command: Command, kuutamo_node: Kuutamod, msg: str) -> bool:
    """check Kuutamo node keep updating the block height"""

    note(f"##### check blocking for {msg} ######")
    proc = command.run(
        ["neard", "--home", str(kuutamo_node.neard_home), "view-state", "state"],
        stdout=subprocess.PIPE,
    )
    assert proc.stdout is not None
    block_height = int(proc.stdout.read().splitlines()[0].split(" ")[-1])
    note(
        f"{kuutamo_node.node_id}'s block_height before leader terminate: {block_height}"
    )
    time.sleep(10)
    proc = command.run(
        ["neard", "--home", str(kuutamo_node.neard_home), "view-state", "state"],
        stdout=subprocess.PIPE,
    )
    assert proc.stdout is not None
    new_block_height = int(proc.stdout.read().splitlines()[0].split(" ")[-1] or 0)
    note(f"{kuutamo_node.node_id}'s block_height: {new_block_height}")
    if new_block_height >= block_height + 5:
        note(f"{msg} keep blocking")
        return True
    else:
        return False


@work_with_neard_versions(["1.29.0"])
def test_maintenance_shutdown(
    kuutamod: Path,
    kuutamoctl: Path,
    command: Command,
    consul: Consul,
    near_network: NearNetwork,
    ports: Ports,
) -> None:
    kuutamods = []
    for idx in range(2):
        kuutamods.append(
            run_node(
                neard_home=near_network.home / f"kuutamod{idx}",
                kuutamod=kuutamod,
                ports=ports,
                near_network=near_network,
                command=command,
                consul=consul,
            )
        )
    leader = None
    follower = None

    note("wait for leader election to take place")
    while leader is None:
        for idx, k in enumerate(kuutamods):
            res = query_prometheus_endpoint("127.0.0.1", k.exporter_port)
            if res.get('kuutamod_state{type="Validating"}') == "1":
                note(f"leader is kuutamo{idx}")
                leader = kuutamods[idx]
                del kuutamods[idx]
                follower = kuutamods.pop()
                break
            time.sleep(0.1)
    proc = command.run(
        [
            str(kuutamoctl),
            "--json",
            "--consul-url",
            consul.consul_url,
            "active-validator",
        ],
        stdout=subprocess.PIPE,
    )
    assert proc.stdout
    print(proc.stdout)
    data = json.load(proc.stdout)
    assert data.get("ID")
    assert proc.wait() == 0
    assert follower is not None

    # Check if neard processes use correct specified ports
    wait_for_port("127.0.0.1", leader.validator_port)
    wait_for_port("127.0.0.1", follower.voter_port)

    assert len(kuutamods) == 0 and follower is not None
    follower_res = query_prometheus_endpoint("127.0.0.1", follower.exporter_port)
    assert follower_res['kuutamod_state{type="Validating"}'] == "0"

    note("##### test maintenance shutdown on follower ######")
    pid = query_neard_pid("127.0.0.1", follower.exporter_port)
    assert pid is not None

    send_maintenance_shutdown_comand(command, kuutamoctl, follower)
    start = time.perf_counter()

    while True:
        new_pid = query_neard_pid("127.0.0.1", follower.exporter_port)
        if pid != new_pid:
            break
        time.sleep(0.1)
    duration = time.perf_counter() - start
    note(f"------------- Follower restart took {duration}s ---------------")

    note("##### test maintenance shutdown on leader ######")
    pid = query_neard_pid("127.0.0.1", leader.exporter_port)
    assert pid is not None

    send_maintenance_shutdown_comand(command, kuutamoctl, leader)

    note("checking on leader restart and keep producing block")
    check = 0
    while not check_blocking(
        command, leader, "leader restarted after maintenance shutdown"
    ):
        check += 1
        if check > 10:
            note("leader did not restart correctly")
            assert False
    note("------------- Leader restarted ---------------")
