#!/usr/bin/env python3

import time
from pathlib import Path

from command import Command
from consul import Consul
from kuutamod import Kuutamod
from log_utils import Section
from ports import Ports
from setup_localnet import NearNetwork


def test_maintenance_shutdown_metrics(
    kuutamod: Path,
    kuutamoctl: Path,
    command: Command,
    consul: Consul,
    near_network: NearNetwork,
    ports: Ports,
) -> None:
    leader = Kuutamod.run(
        neard_home=near_network.home / "kuutamod0",
        kuutamod=kuutamod,
        ports=ports,
        near_network=near_network,
        command=command,
        kuutamoctl=kuutamoctl,
        consul=consul,
        debug=False,
    )

    with Section("Wait leader validating"):
        while True:
            try:

                res = leader.metrics()
                if res.get('kuutamod_state{type="Validating"}') == "1":
                    break
            except (ConnectionRefusedError, ConnectionResetError):
                pass
            time.sleep(0.1)
        leader.wait_rpc_port()

    # Book a far away block height, so we can check on metric before shutdown
    with Section("test maintenance shutdown metrics"):
        pid = leader.neard_pid()
        assert pid is not None

        proc = leader.execute_command(
            "maintenance-shutdown",
            "--shutdown-at",
            "1000",
        )
        assert proc.returncode == 0
        new_pid = leader.neard_pid()
        assert new_pid == pid

        for i in range(120):
            try:
                res = leader.neard_metrics()
                if (
                    res.get("near_block_expected_shutdown") == "1000"
                    and res.get("near_dynamic_config_changes") == "1"
                ):
                    break
            except (ConnectionRefusedError, ConnectionResetError):
                pass
            time.sleep(0.1)
        else:
            assert (
                res.get("near_block_expected_shutdown") == "1000"
                or res.get("near_dynamic_config_changes") == "1"
            )
