#!/usr/bin/env python3


import os
import json
import time
from pathlib import Path
import pytest

from command import Command
from consul import Consul
from kuutamod import Kuutamod
from ports import Ports
from setup_localnet import NearNetwork
from typing import Any, List
from note import note, Section


def work_with_neard_versions(
    versions: List[str],
) -> Any:
    return pytest.mark.skipif(
        os.environ.get("NEARD_VERSION") not in versions,
        reason=f"Not suitable neard for current test, this test only for {versions}",
    )


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
            Kuutamod.run(
                neard_home=near_network.home / f"kuutamod{idx}",
                kuutamod=kuutamod,
                ports=ports,
                near_network=near_network,
                command=command,
                kuutamoctl=kuutamoctl,
                consul=consul,
            )
        )
    leader = None
    follower = None

    with Section("leader election"):
        while leader is None:
            for idx, k in enumerate(kuutamods):
                res = k.metrics()
                if res.get('kuutamod_state{type="Validating"}') == "1":
                    note(f"leader is kuutamo{idx}")
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

    with Section("test maintenance shutdown on follower"):
        pid = follower.neard_pid()
        assert pid is not None

        proc = follower.execute_command(
            "maintenance-shutdown",
            "1",  # Use one block window for maintenance shutdown in test
        )
        assert proc.returncode == 0

        start = time.perf_counter()

        while True:
            new_pid = follower.neard_pid()
            if pid != new_pid:
                break
            time.sleep(0.1)
        duration = time.perf_counter() - start
        note(f"follower restart time {duration}")

    with Section("test maintenance shutdown on leader"):
        pid = leader.neard_pid()
        assert pid is not None

        proc = leader.execute_command(
            "maintenance-shutdown",
            "1",  # Use one block window for maintenance shutdown in test
        )

        assert proc.returncode == 0
        for i in range(5):
            new_pid = leader.neard_pid()
            if new_pid is not pid:
                break
        else:
            assert new_pid is not pid

        # TODO check metric in other test, this will fail if the leader restart really quick
        # for i in range(15):
        #     time.sleep(1)
        #     try:
        #         res = leader.neard_metrics()
        #         if (
        #             res.get("near_block_expected_shutdown") is not None
        #             or res.get("near_dynamic_config_changes") is not None
        #         ):
        #             break
        #     except ConnectionRefusedError:
        #         continue
        # else:
        #     assert (
        #         res.get("near_block_expected_shutdown") is not None
        #         or res.get("near_dynamic_config_changes") is not None
        #     )

        note("checking on leader restart and keep producing block")
        check = 0
        while not leader.check_blocking():
            check += 1
            if check > 3:
                note("leader did not restart correctly")
                assert False
