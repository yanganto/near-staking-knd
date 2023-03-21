#!/usr/bin/env python3

import os
import subprocess
from pathlib import Path
from typing import Optional

import pytest


class Cargo:
    def __init__(self, project_root: Path):
        self.project_root = project_root

    def build(self, target: str = "debug") -> Path:
        env = os.environ.copy()
        extra_flags = []
        if target == "release":
            extra_flags += ["--release"]
        if not os.environ.get("TEST_NO_REBUILD"):
            subprocess.run(
                ["cargo", "build"] + extra_flags,
                cwd=self.project_root,
                env=env,
                check=True,
            )
        return self.project_root.joinpath("target", target)


@pytest.fixture
def cargo(project_root: Path) -> Cargo:
    return Cargo(project_root)


BUILD: Optional[Path] = None
RELEASE_BUILD = os.environ.get("KUUTAMOD_BIN")


@pytest.fixture
def kneard(cargo: Cargo) -> Path:
    global BUILD
    global RELEASE_BUILD
    if RELEASE_BUILD is not None:
        return Path(RELEASE_BUILD).joinpath("kneard")

    if BUILD is None:
        BUILD = cargo.build()
    return BUILD.joinpath("kneard")


@pytest.fixture
def kuutamoctl(cargo: Cargo) -> Path:
    global BUILD
    global RELEASE_BUILD
    if RELEASE_BUILD is not None:
        return Path(RELEASE_BUILD).joinpath("kuutamoctl")

    if BUILD is None:
        BUILD = cargo.build()
    return BUILD.joinpath("kuutamoctl")
