#!/usr/bin/env python3


import json
import os
import time
from pathlib import Path
from typing import Any, List

import pytest
from command import Command
from consul import Consul
from kuutamod import Kuutamod
from log_utils import Section, log_note
from ports import Ports
from setup_localnet import NearNetwork


def work_with_neard_versions(
    versions: List[str],
) -> Any:
    return pytest.mark.skipif(
        os.environ.get("NEARD_VERSION") not in versions,
        reason=f"Not suitable neard for current test, this test only for {versions}",
    )


def test_maintenance_restart(
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
                debug=False,
            )
        )
    leader = None
    follower = None

    with Section("leader election"):
        while leader is None:
            for idx, k in enumerate(kuutamods):
                res = k.metrics()
                if res.get('kuutamod_state{type="Validating"}') == "1":
                    log_note(f"leader is kuutamo{idx}")
                    leader = kuutamods[idx]
                    del kuutamods[idx]
                    follower = kuutamods.pop()
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

        assert len(kuutamods) == 0 and follower is not None
        follower_res = follower.metrics()
        assert follower_res['kuutamod_state{type="Validating"}'] == "0"

    with Section("test maintenance restart on follower"):
        follower.wait_rpc_port()
        pid = follower.neard_pid()
        assert pid is not None

        proc = follower.execute_command(
            "maintenance-restart",
            "1",  # Use one block window for maintenance restart in test
        )
        assert proc.returncode == 0

        start = time.perf_counter()

        while True:
            new_pid = follower.neard_pid()
            if pid != new_pid:
                break
            time.sleep(0.1)
        duration = time.perf_counter() - start
        log_note(f"follower restart time {duration}")

    with Section("test maintenance status for restart"):
        leader.wait_rpc_port()
        pid = leader.neard_pid()
        assert pid is not None

        proc = leader.execute_command(
            "maintenance-restart",
            "--shutdown-at",
            "1000",
        )
        assert proc.returncode == 0
        proc = leader.execute_command("maintenance-status")
        log_note(proc.stdout)
        assert "restart" in proc.stdout

    with Section("test cancel for maintenance restart"):
        proc = leader.execute_command(
            "maintenance-restart",
            "--cancel",
        )

        for i in range(150):
            try:
                res = leader.neard_metrics()
                if (
                    res.get("near_block_expected_shutdown")
                    == "0"  # no block height for shutdown
                    and res.get("near_config_reloads_total")
                    == "3"  # the second time dynamic config change
                ):
                    break
            except (ConnectionRefusedError, ConnectionResetError):
                pass
            time.sleep(0.1)
        else:
            assert (
                res.get("near_block_expected_shutdown") == "0"
                and res.get("near_config_reloads_total") == "3"
            )

        proc = leader.execute_command("maintenance-status")
        log_note(proc.stdout)
        assert "no maintenance setting now" in proc.stdout

    with Section("test maintenance restart on leader"):
        proc = leader.execute_command(
            "maintenance-restart",
            "1",  # Use one block window for maintenance restart in test
        )

        assert proc.returncode == 0
        for i in range(600):
            new_pid = leader.neard_pid()
            if new_pid != pid:
                break
            time.sleep(0.1)
        else:
            assert new_pid != pid

        log_note("checking on leader restart and keep producing block")
        assert leader.network_produces_blocks()
