#!/usr/bin/env python3

from dataclasses import dataclass
from pathlib import Path
from tempfile import TemporaryDirectory
import subprocess
from typing import Iterator, Optional
import time

import pytest
import json
from command import Command
from ports import Ports
from network import wait_for_port


@dataclass
class Consul:
    dns_port: int
    grpc_port: int
    http_port: int
    serv_lan_port: int
    serv_wan_port: int
    server_port: int
    management_token: Optional[str] = None

    @property
    def consul_url(self) -> str:
        """
        Returns url to consul's http API
        """
        return f"http://localhost:{self.http_port}"


def _consul(
    command: Command, ports: Ports, test_root: Path, enable_acl: bool = False
) -> Iterator[Consul]:
    with TemporaryDirectory() as dir:
        consul_data = Path(dir) / "consul"
        start_port = ports.allocate(6)
        http_port = start_port + 2
        cmd = [
            "consul",
            "agent",
            "-server",
            "-bootstrap-expect",
            "1",
            "-bind",
            "127.0.0.1",
            f"-dns-port={start_port}",
            f"-grpc-port={start_port + 1}",
            f"-http-port={http_port}",
            f"-serf-lan-port={start_port + 3}",
            f"-serf-wan-port={start_port + 4}",
            f"-server-port={start_port + 5}",
            "-data-dir",
            str(consul_data),
            "-log-level=warn",
        ]
        if enable_acl:
            cmd += ["-config-file", str(test_root / "consul-acl.hcl")]

        command.run(cmd)
        management_token = None
        if enable_acl:
            wait_for_port("127.0.0.1", http_port)
            while True:
                out = subprocess.run(
                    [
                        "consul",
                        "acl",
                        "bootstrap",
                        "-format",
                        "json",
                        "-http-addr",
                        f"http://127.0.0.1:{http_port}",
                    ],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                if out.returncode == 1:
                    if b"No cluster leader" in out.stderr:
                        time.sleep(0.1)
                        continue
                    else:
                        assert out.returncode != 0, "consul acl bootstrap failed"
                else:
                    data = json.loads(out.stdout)
                    management_token = data["SecretID"]
                    break
        p = range(start_port, start_port + 6)
        yield Consul(
            dns_port=p[0],
            grpc_port=p[1],
            http_port=p[2],
            serv_lan_port=p[3],
            serv_wan_port=p[4],
            server_port=p[5],
            management_token=management_token,
        )


@pytest.fixture
def consul(command: Command, ports: Ports, test_root: Path) -> Iterator[Consul]:
    yield from _consul(command, ports, test_root)


@pytest.fixture
def consul_with_acls(
    command: Command, ports: Ports, test_root: Path
) -> Iterator[Consul]:
    yield from _consul(command, ports, test_root, enable_acl=True)
