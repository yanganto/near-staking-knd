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
        command.run(
            ["neard", "--home", str(neard_home / "node0"), "run"],
            extra_env=dict(RUST_LOG="info"),
        )
        command.run(
            ["neard", "--home", str(neard_home / "node1"), "run", "--boot-nodes", node],
            extra_env=dict(RUST_LOG="warn"),
        )
        command.run(
            ["neard", "--home", str(neard_home / "node2"), "run", "--boot-nodes", node],
            extra_env=dict(RUST_LOG="warn"),
        )
        # node3 serves as our validator key just now.
        yield near_network
