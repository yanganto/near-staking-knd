from __future__ import annotations
from dataclasses import dataclass
from pathlib import Path
from subprocess import Popen
from typing import Optional
import http.client
import json
import subprocess
import time

from command import Command
from consul import Consul
from network import wait_for_port
from ports import Ports
from setup_localnet import NearNetwork
from prometheus import query_prometheus_endpoint


kuutamoctl: Optional[Path] = None


def set_kuutamoctl(path: Path) -> None:
    """Set up kuutamo ctl"""
    global kuutamoctl
    kuutamoctl = path


@dataclass
class Kuutamod:
    proc: Popen
    exporter_port: int
    validator_port: int
    voter_port: int
    rpc_port: int
    node_id: str
    control_socket: Path
    neard_home: Path
    command: Command

    @classmethod
    def run(
        cls,
        neard_home: Path,
        consul: Consul,
        kuutamod: Path,
        command: Command,
        ports: Ports,
        near_network: NearNetwork,
    ) -> Kuutamod:
        exporter_port = ports.allocate(3)
        validator_port = exporter_port + 1
        voter_port = exporter_port + 2
        validator_key = near_network.home / "node3" / "validator_key.json"
        validator_node_key = near_network.home / "node3" / "node_key.json"
        voter_node_key = neard_home / "voter_node_key.json"
        node_id = str(neard_home.name)
        env = dict(
            KUUTAMO_CONSUL_URL=f"http://127.0.0.1:{consul.http_port}",
            KUUTAMO_NODE_ID=str(neard_home.name),
            KUUTAMO_EXPORTER_ADDRESS=f"127.0.0.1:{exporter_port}",
            KUUTAMO_VALIDATOR_NETWORK_ADDR=f"127.0.0.1:{validator_port}",
            KUUTAMO_VOTER_NETWORK_ADDR=f"127.0.0.1:{voter_port}",
            KUUTAMO_VALIDATOR_KEY=str(validator_key),
            KUUTAMO_VALIDATOR_NODE_KEY=str(validator_node_key),
            KUUTAMO_VOTER_NODE_KEY=str(voter_node_key),
            KUUTAMO_NEARD_HOME=str(neard_home),
            KUUTAMO_NEARD_BOOTNODES=near_network.boostrap_node,
            RUST_BACKTRACE="1",
        )
        config = json.load(open(neard_home / "config.json"))
        proc = command.run([str(kuutamod)], extra_env=env)
        wait_for_port("127.0.0.1", exporter_port)

        return cls(
            proc=proc,
            exporter_port=exporter_port,
            node_id=node_id,
            validator_port=validator_port,
            voter_port=voter_port,
            control_socket=neard_home / "kuutamod.ctl",
            neard_home=neard_home,
            command=command,
            rpc_port=int(config["rpc"]["addr"].split(":")[-1]),
        )

    def neard_pid(self) -> Optional[int]:
        """Query pid for neard which managed by the kuutamod with 3 times retry"""
        for i in range(3):
            try:
                conn = http.client.HTTPConnection("127.0.0.1", self.exporter_port)
                conn.request("GET", "/neard-pid")
                response = conn.getresponse()
                body = response.read().decode("utf-8")
                if body == "":
                    return None
                return int(body)
            except ConnectionRefusedError:
                if i == 2:
                    raise ConnectionRefusedError
                pass
            time.sleep(i)
        return None

    def metrics(self) -> dict:
        """Query the prometheus metrics for kuutamod"""
        return query_prometheus_endpoint("127.0.0.1", self.exporter_port)

    def neard_metrics(self) -> dict:
        """Query the prometheus metrics for neard which managed by the kuutamod"""
        return query_prometheus_endpoint("127.0.0.1", self.rpc_port)

    def wait_validator_port(self) -> None:
        """Wait validator port"""
        wait_for_port("127.0.0.1", self.validator_port)

    def wait_voter_port(self) -> None:
        """Wait validator port"""
        wait_for_port("127.0.0.1", self.voter_port)

    def terminate(self) -> None:
        """Terminate kuutamod processes"""
        self.proc.terminate()

    def wait(self) -> None:
        """Wait kuutamod processes"""
        self.proc.wait()

    def check_blocking(self) -> bool:
        """check Kuutamo node keep updating the block height"""

        proc = self.command.run(
            ["neard", "--home", str(self.neard_home), "view-state", "state"],
            stdout=subprocess.PIPE,
        )
        assert proc.stdout is not None
        block_height = int(proc.stdout.read().splitlines()[0].split(" ")[-1])
        time.sleep(10)
        proc = self.command.run(
            ["neard", "--home", str(self.neard_home), "view-state", "state"],
            stdout=subprocess.PIPE,
        )
        assert proc.stdout is not None
        new_block_height = int(proc.stdout.read().splitlines()[0].split(" ")[-1] or 0)
        if new_block_height >= block_height + 5:
            return True
        else:
            return False

    def execute_command(self, *args: str) -> None:
        """Send command to Kuutamod with 5 times retry"""
        global kuutamoctl

        assert kuutamoctl is not None, "please set_kuutamoctl before send_command"

        for i in range(5):
            proc = self.command.run(
                [str(kuutamoctl), "--control-socket", str(self.control_socket), *args],
                stdout=subprocess.PIPE,
            )
            proc.wait()
            if proc.returncode == 0:
                return
            time.sleep(i)
        assert False
