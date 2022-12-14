#!/usr/bin/env python3

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.joinpath("tests")))

import os

from setup_localnet import setup_network_config

PROJECT_ROOT = Path(__file__).parent.parent.resolve()


def main() -> None:
    near_home = Path(
        os.environ.get("NEAR_HOME", PROJECT_ROOT.joinpath(".data/near/localnet"))
    )
    if near_home.exists():
        return
    setup_network_config(near_home, start_port=33300)


if __name__ == "__main__":
    main()
