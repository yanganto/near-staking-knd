#!/usr/bin/env python3

import json
import os
import time
from pathlib import Path
from signal import SIGKILL

from command import Command
from consul import Consul
from kneard import Kuutamod
from ports import Ports
from setup_localnet import NearNetwork


def test_multiple_nodes(
    kneard: Path,
    kneard_ctl: Path,
    command: Command,
    consul: Consul,
    near_network: NearNetwork,
    ports: Ports,
) -> None:
    # Uncomment this to save config and logs
    # near_network.set_artifact_path("/tmp/test.tgz")

    # FIXME Just now we use the validator key of genesis node3 for our setup
    kneards = []
    for idx in range(2):
        kneards.append(
            Kuutamod.run(
                neard_home=near_network.home / f"kneard{idx}",
                kneard=kneard,
                ports=ports,
                near_network=near_network,
                command=command,
                consul=consul,
                kneard_ctl=kneard_ctl,
                debug=True,
            )
        )
    leader = None
    follower = None
    # wait for leader election to take place
    while leader is None:
        for idx, k in enumerate(kneards):
            res = k.metrics()
            print(idx, res)
            if res.get('kneard_state{type="Validating"}') == "1":
                leader = kneards[idx]
                del kneards[idx]
                follower = kneards.pop()
                break
            time.sleep(0.1)
    proc = leader.execute_command("--json", "active-validator")
    assert proc.stdout
    print(proc.stdout)
    data = json.loads(proc.stdout)
    assert data.get("Node")
    assert follower is not None

    # Check if neard processes use correct specified ports
    leader.wait_validator_port()
    follower.wait_voter_port()

    assert len(kneards) == 0 and follower is not None
    follower_res = follower.metrics()
    assert follower_res['kneard_state{type="Validating"}'] == "0"

    assert leader.network_produces_blocks()
    assert follower.network_produces_blocks()

    print("##### test crash ######")
    pid = leader.neard_pid()
    assert pid is not None
    os.kill(pid, SIGKILL)
    start = time.perf_counter()
    while True:
        res = follower.metrics()
        if res.get('kneard_state{type="Validating"}') == "1":
            break
        print(res)
        time.sleep(0.1)
    duration = time.perf_counter() - start
    print(f"------------- Failover took {duration}s ---------------")
    assert follower.network_produces_blocks()
    assert leader.network_produces_blocks()
    leader, follower = follower, leader

    while True:
        res = follower.metrics()
        if res.get('kneard_state{type="Voting"}') == "1":
            break
        print(res)
        time.sleep(0.1)

    print("##### test graceful failover ######")
    # graceful migration
    leader.terminate()
    start = time.perf_counter()
    while True:
        res = follower.metrics()
        if res.get('kneard_state{type="Validating"}') == "1":
            break
        print(res)
        time.sleep(0.1)
    duration = time.perf_counter() - start
    print(f"------------- Failover took {duration}s ---------------")
    leader.wait()
    assert follower.network_produces_blocks()
