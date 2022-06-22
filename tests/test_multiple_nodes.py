#!/usr/bin/env python3

import json
import os
from signal import SIGKILL
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


@dataclass
class Kuutamod:
    proc: Popen
    exporter_port: int
    validator_port: int
    voter_port: int
    node_id: str


def query_neard_pid(host: str, port: int) -> Optional[int]:
    conn = http.client.HTTPConnection(host, port)
    conn.request("GET", "/neard-pid")
    response = conn.getresponse()
    body = response.read().decode("utf-8")
    if body == "":
        return None
    return int(body)


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
    )


def test_multiple_nodes(
    kuutamod: Path,
    kuutamoctl: Path,
    command: Command,
    consul: Consul,
    near_network: NearNetwork,
    ports: Ports,
) -> None:
    # FIXME Just now we use the validator key of genesis node3 for our setup
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
    # wait for leader election to take place
    while leader is None:
        for idx, k in enumerate(kuutamods):
            res = query_prometheus_endpoint("127.0.0.1", k.exporter_port)
            print(idx, res)
            if res.get('kuutamod_state{type="Validating"}') == "1":
                leader = kuutamods[idx]
                del kuutamods[idx]
                follower = kuutamods.pop()
                break
            time.sleep(0.1)
    proc = command.run(
        [str(kuutamoctl), "--consul-url", consul.consul_url, "show-validator"],
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

    print("##### test crash ######")
    pid = query_neard_pid("127.0.0.1", leader.exporter_port)
    assert pid is not None
    os.kill(pid, SIGKILL)
    start = time.perf_counter()
    while True:
        res = query_prometheus_endpoint("127.0.0.1", follower.exporter_port)
        if res.get('kuutamod_state{type="Validating"}') == "1":
            break
        print(res)
        time.sleep(0.1)
    duration = time.perf_counter() - start
    print(f"------------- Failover took {duration}s ---------------")
    leader, follower = follower, leader

    while True:
        res = query_prometheus_endpoint("127.0.0.1", follower.exporter_port)
        if res.get('kuutamod_state{type="Voting"}') == "1":
            break
        print(res)
        time.sleep(0.1)

    print("##### test graceful failover ######")
    # gracefull migration
    leader.proc.terminate()
    start = time.perf_counter()
    while True:
        res = query_prometheus_endpoint("127.0.0.1", follower.exporter_port)
        if res.get('kuutamod_state{type="Validating"}') == "1":
            break
        print(res)
        time.sleep(0.1)
    duration = time.perf_counter() - start
    print(f"------------- Failover took {duration}s ---------------")
    leader.proc.wait()
