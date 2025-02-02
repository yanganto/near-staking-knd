#!/usr/bin/env python3

import tempfile
from typing import IO, Iterator

import pytest


@pytest.fixture
def temporary_file() -> Iterator[IO[str]]:
    """
    Creates a temporary file
    """
    with tempfile.NamedTemporaryFile(mode="w+") as fp:
        yield fp
