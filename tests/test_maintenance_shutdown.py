#!/usr/bin/env python3

import time
from pathlib import Path

from command import Command
from consul import Consul
from kuutamod import Kuutamod
from log_utils import Section, log_note
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

    leader.wait_metrics_port()

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

        for i in range(150):
            try:
                res = leader.neard_metrics()
                if res.get("near_block_expected_shutdown") == "1000" and (
                    res.get("near_dynamic_config_changes") == "1"
                    or res.get("near_config_reloads_total") == "2"
                ):  # the first time dynamic config change
                    break
            except (ConnectionRefusedError, ConnectionResetError):
                pass
            time.sleep(0.1)
        else:
            assert res.get("near_block_expected_shutdown") == "1000" and (
                res.get("near_dynamic_config_changes") == "1"
                or res.get("near_config_reloads_total") == "2"
            )

    with Section("test maintenance status for shutdown"):

        proc = leader.execute_command("maintenance-status")
        log_note(proc.stdout)
        assert "shutdown" in proc.stdout

        pid = leader.neard_pid()
        assert pid is not None

    with Section("test cancel maintenance shutdown"):
        proc = leader.execute_command(
            "maintenance-shutdown",
            "--cancel",
        )
        assert proc.returncode == 0
        new_pid = leader.neard_pid()
        assert new_pid == pid

        for i in range(150):
            try:
                res = leader.neard_metrics()
                if res.get(
                    "near_block_expected_shutdown"
                ) == "0" and (  # no block height for shutdown
                    res.get("near_dynamic_config_changes") == "2"
                    or res.get("near_config_reloads_total") == "3"
                ):  # the second time dynamic config change
                    break
            except (ConnectionRefusedError, ConnectionResetError):
                pass
            time.sleep(0.1)
        else:
            assert res.get("near_block_expected_shutdown") == "0" and (
                res.get("near_dynamic_config_changes") == "2"
                or res.get("near_config_reloads_total") == "3"
            )

        proc = leader.execute_command("maintenance-status")
        log_note(proc.stdout)
        assert "no maintenance setting now" in proc.stdout

    with Section("test kuutamod shutdown with neard"):
        pid = leader.neard_pid()
        assert pid is not None

        proc = leader.execute_command(
            "maintenance-shutdown",
            "1",  # Use one block window for maintenance shutdown in test
        )
        assert proc.returncode == 0
        for i in range(100):
            if not leader:
                break
            time.sleep(0.1)
        else:
            assert not leader
