#!/usr/bin/env python3

from pathlib import Path
from signal import SIGKILL
import json
import os
import time

from command import Command
from consul import Consul
from kuutamod import Kuutamod
from ports import Ports
from setup_localnet import NearNetwork


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
            Kuutamod.run(
                neard_home=near_network.home / f"kuutamod{idx}",
                kuutamod=kuutamod,
                ports=ports,
                near_network=near_network,
                command=command,
                consul=consul,
                kuutamoctl=kuutamoctl,
            )
        )
    leader = None
    follower = None
    # wait for leader election to take place
    while leader is None:
        for idx, k in enumerate(kuutamods):
            res = k.metrics(starting=True)
            print(idx, res)
            if res.get('kuutamod_state{type="Validating"}') == "1":
                leader = kuutamods[idx]
                del kuutamods[idx]
                follower = kuutamods.pop()
                break
            time.sleep(0.1)
    proc = leader.execute_command("--json", "active-validator")
    assert proc.stdout
    print(proc.stdout)
    data = json.loads(proc.stdout)
    assert data.get("ID")
    assert follower is not None

    # Check if neard processes use correct specified ports
    leader.wait_validator_port()
    follower.wait_voter_port()

    assert len(kuutamods) == 0 and follower is not None
    follower_res = follower.metrics()
    assert follower_res['kuutamod_state{type="Validating"}'] == "0"

    assert leader.check_blocking()
    assert follower.check_blocking()

    print("##### test crash ######")
    pid = leader.neard_pid()
    assert pid is not None
    os.kill(pid, SIGKILL)
    start = time.perf_counter()
    while True:
        res = follower.metrics()
        if res.get('kuutamod_state{type="Validating"}') == "1":
            break
        print(res)
        time.sleep(0.1)
    duration = time.perf_counter() - start
    print(f"------------- Failover took {duration}s ---------------")
    assert follower.check_blocking()
    assert leader.check_blocking()
    leader, follower = follower, leader

    while True:
        res = follower.metrics()
        if res.get('kuutamod_state{type="Voting"}') == "1":
            break
        print(res)
        time.sleep(0.1)

    print("##### test graceful failover ######")
    # gracefull migration
    leader.terminate()
    start = time.perf_counter()
    while True:
        res = follower.metrics()
        if res.get('kuutamod_state{type="Validating"}') == "1":
            break
        print(res)
        time.sleep(0.1)
    duration = time.perf_counter() - start
    print(f"------------- Failover took {duration}s ---------------")
    leader.wait()
    assert follower.check_blocking()
