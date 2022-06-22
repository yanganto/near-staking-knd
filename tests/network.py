#!/usr/bin/env python3

import socket
import subprocess
import time
from typing import Optional
from timeit import default_timer as timer


def wait_for_port(
    host: str, port: int, timeout: int = 40, proc: Optional[subprocess.Popen] = None
) -> None:
    """
    Wait for a tcp port to reachable. Optionally also checks if a server process is still alive
    """
    start = timer()

    while True:
        try:
            with socket.create_connection((host, port), timeout=1):
                return
        except OSError:
            if proc is not None:
                res = proc.poll()
                assert res is None, f"Our server process exited with {res}"
            time.sleep(0.01)
            end = timer()
            if end - start > timeout:
                raise Exception(f"timeout waiting for {host}:{port}")
