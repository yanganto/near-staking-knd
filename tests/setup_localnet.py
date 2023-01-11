#!/usr/bin/env python3

import io
import json
import os
import shutil
import subprocess
import sys
import tarfile
from dataclasses import dataclass
from pathlib import Path
from shlex import quote
from typing import IO, Any, Callable, Dict, List, Optional, Text, Union

from log_utils import log_note

_FILE = Union[None, int, IO[Any]]

HAS_TTY = sys.stderr.isatty()


def color_text(code: int, file: IO[Any] = sys.stdout) -> Callable[[str], None]:
    """
    Print with color if stderr is a tty
    """

    def wrapper(text: str) -> None:
        if HAS_TTY:
            print(f"\x1b[{code}m{text}\x1b[0m", file=file)
        else:
            print(text, file=file)

    return wrapper


warn = color_text(91, file=sys.stderr)
info = color_text(92, file=sys.stderr)


def run(
    cmd: List[str],
    extra_env: Dict[str, str] = {},
    stdin: _FILE = None,
    stdout: _FILE = None,
    stderr: _FILE = None,
    input: Optional[str] = None,
    timeout: Optional[int] = None,
    check: bool = True,
    shell: bool = False,
) -> "subprocess.CompletedProcess[Text]":
    """
    Run a program while also pretty print the command that it runs
    """
    env = os.environ.copy()
    env.update(extra_env)
    env_string = []
    for k, v in extra_env.items():
        env_string.append(f"{k}={v}")
    pretty_cmd = "$ "
    if input is not None:
        pretty_cmd += f"echo {quote(input)} |"
    if len(extra_env) > 0:
        pretty_cmd += " ".join(env_string) + " "
    if shell:
        pretty_cmd += "sh -c "
    pretty_cmd += " ".join(map(quote, cmd))
    if isinstance(stdin, io.IOBase):
        pretty_cmd += f" < {stdin.name}"
    if isinstance(stdout, io.IOBase):
        pretty_cmd += f" > {stdout.name}"
    if isinstance(stderr, io.IOBase):
        pretty_cmd += f" 2> {stderr.name}"
    info(pretty_cmd)
    return subprocess.run(
        cmd[0] if shell else cmd,
        stdin=stdin,
        stdout=stdout,
        stderr=stderr,
        timeout=timeout,
        check=check,
        env=env,
        text=True,
        input=input,
        shell=shell,
    )


# From nearcore/nearcore/src/config.rs
ONE_NEAR = 10**24
TESTING_INIT_BALANCE = 1_000_000_000 * ONE_NEAR
# Where does this come from? I took it from genesis.json
TEST_CODE_HASH = "11111111111111111111111111111111"


def write_near_key(home: Path, account_id: str) -> None:
    run(
        [
            "neard",
            "--home",
            str(home.joinpath(account_id)),
            "init",
            "--account-id",
            account_id,
        ]
    )


def setup_kuutamod_home(home: Path) -> None:
    write_near_key(home.parent, home.name)
    # Delete validator keys as those are provided by neard!
    (home / "validator_key.json").unlink()
    # kuutamod will either place voter_node_key or the validator node key at node_key.json
    (home / "node_key.json").rename(home / "voter_node_key.json")


def setup_additional_keys(near_home: Path, num_nodes: int) -> None:
    write_near_key(near_home, "owner")
    write_near_key(near_home, "validator")

    setup_kuutamod_home(near_home / "kuutamod0")
    setup_kuutamod_home(near_home / "kuutamod1")

    genesis_path = near_home / "node0" / "genesis.json"

    genesis = json.loads(genesis_path.read_text())
    account = dict(
        amount=str(TESTING_INIT_BALANCE),
        locked=0,
        storage_usage=0,
        code_hash=TEST_CODE_HASH,
        version="V1",
    )
    account = dict(account_id="owner", account=account)

    path = near_home / "owner" / "validator_key.json"
    node_key = json.loads(path.read_text())

    access_key = dict(
        account_id="owner",
        public_key=node_key["public_key"],
        access_key=dict(nonce=0, permission="FullAccess"),
    )
    genesis["records"].append(dict(Account=account))
    genesis["records"].append(dict(AccessKey=access_key))
    genesis["total_supply"] = str(int(genesis["total_supply"]) + TESTING_INIT_BALANCE)

    genesis_content = json.dumps(genesis, indent=4, sort_keys=True)
    for i in range(num_nodes):
        (near_home / f"node{i}" / "genesis.json").write_text(genesis_content)

    (near_home / "kuutamod0" / "genesis.json").write_text(genesis_content)
    (near_home / "kuutamod1" / "genesis.json").write_text(genesis_content)


@dataclass
class NearNode:
    rpc_port: int
    network_port: int


@dataclass
class NearNetwork:
    home: Path
    nodes: List[NearNode]
    artifact_path: Optional[Path]

    @property
    def bootstrap_node(self) -> str:
        node0_key = json.loads((self.home / "node0" / "node_key.json").read_text())
        return f"{node0_key['public_key']}@127.0.0.1:{self.nodes[0].network_port}"

    def save_artifacts(self) -> None:
        """tar and gzip all logs into artifact"""
        if self.artifact_path:
            tarball = tarfile.open(self.artifact_path, "w:gz")
            for root, dirs, files in os.walk(self.home):
                for f in files:
                    # NOTE: 'json' for config, 'txt' for neard log, 'log' for rocksdb log
                    if f.split(".")[-1] in ("json", "txt"):
                        log_note(str(os.path.join(root, f)))
                        tarball.add(os.path.join(root, f))
            tarball.close()

    def set_artifact_path(self, path: Path) -> None:
        self.artifact_path = path


def setup_network_config(near_home: Path, start_port: int) -> NearNetwork:
    near_tmp = near_home.parent / "localnet-tmp"
    if near_tmp.exists():
        shutil.rmtree(near_tmp)
    # FIXME: Instead of setting up pool contracts etc, right now we just set up
    # an additional node, which we use as the validator key for our fail-over setup
    num_nodes = 4
    num_kuutamo_nodes = 2
    run(
        [
            "neard",
            "--home",
            str(near_tmp),
            "localnet",
            "--shards",
            "1",
            "--v",
            str(num_nodes),
        ]
    )

    setup_additional_keys(near_tmp, num_nodes)

    nodes = []
    with open(near_tmp.joinpath("nodes"), "w") as f:
        node_names = [f"node{i}" for i in range(num_nodes)]
        node_names.extend([f"kuutamod{i}" for i in range(num_kuutamo_nodes)])
        for i, name in enumerate(node_names):
            path = near_tmp / name / "config.json"

            data = json.loads(path.read_text())
            rpc_port = start_port + i * 2
            data["store"] = {"max_open_files": 512}
            data["network"]["allow_private_ip_in_public_addrs"] = True
            data["rpc"]["addr"] = f"0.0.0.0:{rpc_port}"
            # change to track_all_shards true after this issue solved
            # https://github.com/near/nearcore/issues/4930
            data["tracked_shards"] = [0]
            network = data["network"]
            network["addr"] = f"0.0.0.0:{rpc_port + 1}"
            # this makes debugging tests a bit more pleasant and less spammy
            network["peer_stats_period"]["secs"] = 15
            nodes.append(NearNode(rpc_port=rpc_port, network_port=rpc_port + 1))
            f.write(
                f"{name}: rpc 127.0.0.1:{rpc_port} network 127.0.0.1:{rpc_port + 1}\n"
            )
            path.write_text(json.dumps(data, indent=2))

    near_tmp.rename(near_home)
    return NearNetwork(near_home, nodes, None)


def local_near(
    near_home: Path, near_port: int, args: List[str], input: Optional[str] = None
) -> None:
    # direnv sets PROJ_ROOT
    extra_env = dict(
        NEAR_ENV="local",
        NEAR_CLI_LOCALNET_RPC_SERVER_URL=f"http://localhost:{near_port}",
    )
    run(
        ["near", "--keyPath", str(near_home / "owner/validator_key.json")] + args,
        extra_env=extra_env,
        input=input,
    )


def deploy_contract(
    near_home: Path,
    near_port: int,
    master_account_id: str,
    account_id: str,
    cost: int,
    contract: Path,
    args: str,
) -> None:
    code = f"""
await new Promise(resolve => setTimeout(resolve, 100));
const fs = require('fs');
const account = await near.account("{master_account_id}");
const contractName = "{account_id}";
const newArgs = {args};
await account.signAndSendTransaction(
    contractName,
    [
        nearAPI.transactions.createAccount(),
        nearAPI.transactions.transfer("{cost}"),
        nearAPI.transactions.deployContract(fs.readFileSync("{contract}")),
        nearAPI.transactions.functionCall("new", Buffer.from(JSON.stringify(newArgs)), 10000000000000, "0"),
    ]);
    """
    local_near(near_home, near_port, ["repl"], input=code)


def setup_core_contracts(near_home: Path, node_port: int = 33301) -> None:
    contracts = os.environ.get("CORE_CONTRACTS")
    if contracts is None:
        raise Exception("CORE_CONTRACTS is not set")
    contract_path = Path(contracts)

    deploy_contract(
        near_home,
        node_port,
        "owner",
        "transfer-vote.owner",
        15 * ONE_NEAR,
        contract_path / "voting/res/voting_contract.wasm",
        args="{}",
    )
    deploy_contract(
        near_home,
        node_port,
        "owner",
        "lockup-whitelist.owner",
        15 * ONE_NEAR,
        contract_path / "whitelist/res/whitelist.wasm",
        # TODO: create more accounts for each contract
        args='{foundation_account_id: "owner"}',
    )
    deploy_contract(
        near_home,
        node_port,
        "owner",
        "poolv1.owner",
        15 * ONE_NEAR,
        contract_path / "staking-pool-factory/res/staking_pool_factory.wasm",
        # TODO: create more accounts for each contract
        args='{staking_pool_whitelist_account_id: "owner"}',
    )
    code = """
    await new Promise(resolve => setTimeout(resolve, 100));
    const account = await near.account("owner");
    const contractName = "lockup-whitelist.owner";
    const args = {factory_account_id: "poolv1.owner"};
    await account.signAndSendTransaction(
        contractName,
        [
            nearAPI.transactions.functionCall("add_factory", Buffer.from(JSON.stringify(args)), 10000000000000, "0"),
        ]);
    """
    local_near(near_home, node_port, ["repl"], input=code)
