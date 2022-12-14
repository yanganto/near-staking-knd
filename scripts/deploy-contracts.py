#!/usr/bin/env python3

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.joinpath("tests")))

import os

from setup_localnet import setup_core_contracts

PROJECT_ROOT = Path(__file__).parent.parent.resolve()


def main() -> None:
    near_home = Path(
        os.environ.get("NEAR_HOME", PROJECT_ROOT.joinpath(".data/near/localnet"))
    )
    state_file = near_home / "setup-contracts"
    if state_file.exists():
        return
    setup_core_contracts(near_home, node_port=33300)
    state_file.write_text("OK")


if __name__ == "__main__":
    main()
