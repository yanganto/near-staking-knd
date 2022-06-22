import http.client
from typing import Dict


def query_prometheus_endpoint(host: str, port: int) -> Dict[str, str]:
    buffer = b""
    conn = http.client.HTTPConnection(host, port)
    conn.request("GET", "/metrics")
    response = conn.getresponse()
    buffer = response.read()
    lines = buffer.decode("utf-8").split("\n")

    res = {}
    for line in lines:
        if line.startswith("#") or line == "":
            continue
        key, value = line.split(" ", 2)
        res[key] = value
    return res
