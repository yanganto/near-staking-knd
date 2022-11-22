#!/usr/bin/env python3

from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Iterator

import pytest
from command import Command
from ports import Ports
from setup_localnet import NearNetwork, setup_network_config


@pytest.fixture
def near_network(command: Command, ports: Ports) -> Iterator[NearNetwork]:
    """
    Setups a local NEAR network
    """
    with TemporaryDirectory() as dir:
        neard_home = Path(dir) / "neard_home"
        near_network = setup_network_config(neard_home, ports.allocate(6 * 2))
        node = near_network.boostrap_node
        p1 = command.run(
            ["neard", "--home", str(neard_home / "node0"), "run"],
            extra_env=dict(RUST_LOG="info"),
        )
        p2 = command.run(
            ["neard", "--home", str(neard_home / "node1"), "run", "--boot-nodes", node],
            extra_env=dict(RUST_LOG="warn"),
        )
        p3 = command.run(
            ["neard", "--home", str(neard_home / "node2"), "run", "--boot-nodes", node],
            extra_env=dict(RUST_LOG="warn"),
        )
        try:
            # node3 serves as our validator key just now.
            yield near_network
        finally:
            # stop neard processes before removing neard_home
            for p in [p1, p2, p3]:
                try:
                    p.kill()
                except IOError:
                    pass
            near_network.save_artifacts()
