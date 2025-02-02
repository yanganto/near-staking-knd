#!/usr/bin/env python3

import time
from pathlib import Path

from command import Command
from consul import Consul
from kneard import Kuutamod
from log_utils import Section, log_note
from ports import Ports
from setup_localnet import NearNetwork


def test_restart(
    kneard: Path,
    kneard_ctl: Path,
    command: Command,
    consul: Consul,
    near_network: NearNetwork,
    ports: Ports,
) -> None:
    leader = Kuutamod.run(
        neard_home=near_network.home / "kneard0",
        kneard=kneard,
        ports=ports,
        near_network=near_network,
        command=command,
        kneard_ctl=kneard_ctl,
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

    # Book a far away block height, so we can check on metric before neard shutdown
    with Section("test maintenance metrics"):
        pid = leader.neard_pid()
        assert pid is not None

        proc = leader.execute_command(
            "restart",
            "--schedule-at",
            "1000",
        )
        assert proc.returncode == 0
        new_pid = leader.neard_pid()
        assert new_pid == pid

        for i in range(150):
            try:
                res = leader.neard_metrics()
                if (
                    res.get("near_block_expected_shutdown") == "1000"
                    and res.get("near_config_reloads_total") == "2"
                ):
                    break
            except (ConnectionRefusedError, ConnectionResetError):
                pass
            time.sleep(0.1)
        else:
            assert (
                res.get("near_block_expected_shutdown") == "1000"
                and res.get("near_config_reloads_total") == "2"
            )

    with Section("test maintenance status for shutdown"):
        proc = leader.execute_command("maintenance-status")
        log_note(proc.stdout)
        assert "shutdown" in proc.stdout

        pid = leader.neard_pid()
        assert pid is not None

    with Section("test cancel maintenance shutdown"):
        proc = leader.execute_command(
            "restart",
            "--cancel",
        )
        assert proc.returncode == 0
        new_pid = leader.neard_pid()
        assert new_pid == pid

        for i in range(150):
            try:
                res = leader.neard_metrics()
                # no block height for shutdown; and
                # the second time dynamic config change
                if (
                    res.get("near_block_expected_shutdown") == "0"
                    and res.get("near_config_reloads_total") == "3"
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

    with Section("test kneard shutdown with neard"):
        pid = leader.neard_pid()
        assert pid is not None

        proc = leader.execute_command(
            "restart",
            "--wait",
            "1",  # Use one block window for maintenance restart in test
        )
        assert proc.returncode == 0
        assert "shutdown at block height:" in proc.stdout
        assert not leader
