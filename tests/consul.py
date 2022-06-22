#!/usr/bin/env python3

from dataclasses import dataclass
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Iterator

import pytest
from command import Command
from ports import Ports


@dataclass
class Consul:
    dns_port: int
    grpc_port: int
    http_port: int
    serv_lan_port: int
    serv_wan_port: int
    server_port: int

    @property
    def consul_url(self) -> str:
        """
        Returns url to consul's http API
        """
        return f"http://localhost:{self.http_port}"


@pytest.fixture
def consul(command: Command, ports: Ports) -> Iterator[Consul]:
    with TemporaryDirectory() as dir:
        consul_data = Path(dir) / "consul"
        start_port = ports.allocate(6)
        consul = Consul(*range(start_port, start_port + 6))
        command.run(
            [
                "consul",
                "agent",
                "-server",
                "-bootstrap-expect",
                "1",
                "-bind",
                "127.0.0.1",
                f"-dns-port={start_port}",
                f"-grpc-port={start_port + 1}",
                f"-http-port={start_port + 2}",
                f"-serf-lan-port={start_port + 3}",
                f"-serf-wan-port={start_port + 4}",
                f"-server-port={start_port + 5}",
                "-data-dir",
                str(consul_data),
                "-log-level=warn",
            ]
        )
        yield consul
