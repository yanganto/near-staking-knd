#!/usr/bin/env python3


# Neard workload seems to be roughtly 1 main writer + N reader.
#
# Questions asked regarding io access pattern:
#
# https://near.zulipchat.com/#narrow/stream/295302-general/topic/Read.2Fwrite.20ratio.20in.20neard
#
# > @Jörg Thalheim Each column has drastically different access patterns and
# > would require different people to give you an answer on that. See this file
# > for the list of columns:
# > https://github.com/near/nearcore/blob/master/core/store/src/columns.rs
# >
# > For the state column, which stores nodes of the blockchain's patricia
# > merkle tree, I can try to give an answer. All accesses to this column are
# > going through a cache implemented in nearcore. So actually I would not
# > expect many duplicates on the DB level. Also a very specific pattern, we
# > have only reads while processing a chunk, which takes up to 1s, and then we
# > write everything at once when its done. So lots of read-only traffic and
# > then a bulk of writs.  Oh and all our keys there are shard_id ++
# > hash(something). So within a shard, data is spread perfectly bad...   For
# > other columns, I don't even know which of those are heavily used. Maybe
# > @Michał Nazarewicz (mina86) knows, or knows who knows?
#
# https://near.zulipchat.com/#narrow/stream/295302-general/topic/Number.20of.20io.20threads
#
# > 4 readers (1 per each view client thread)
# > 1 writer for the 'client actor'
# > 4 readers (1 per shard) for processing
# > there is another writer in the peer manager for network related queries
# > and another reader (or two) for network graph computation
#
# > Quite recently, we enabled prefetching which has a pool of 8 reader threads
# > per shard, thus 4 * 8 = 32 more threads. How busy these are depends a lot on
# > the workload, though. There will be spikes per shard where these are quite
# > busy but most of the time these are expected to be idle.

# Number of reader threads for DB benchmark, we currently plan in 4 readers for
# view client threads + 4 reader for shard processing and ignore the other
# threads.
NUM_IO_THREADS = 8

import os
import sys
import subprocess
import argparse
import shutil
import resource
from typing import IO, Any, Union, Callable, List, Dict, Optional, Text
from shlex import quote
from pathlib import Path
import io
from dataclasses import dataclass

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


@dataclass
class DbBenchOptions:
    db_path: Path
    num_keys: int
    write_rate_limit: int
    duration: int


def db_bench(opts: DbBenchOptions) -> None:
    common_cmd = [
        "db_bench",
        f"--db={opts.db_path}",
        "--compression_type=zstd",
        # key statistics have been extracted using sst_dump from rocksdb
        "--key_size=40",
        "--value_size=3700",
    ]
    # FIXME figure out what these level man and if we need to change them somewhow
    seed_db = common_cmd + [
        "--compaction_style=1",
        "--num_levels=4",
        "--disable_auto_compactions=1",
        "--benchmarks=fillseqdeterministic",
    ]
    print(f"Initialize rocksdb at {opts.db_path} for benchmark")
    # run(seed_db)

    # FIXME zipfli distributions would be likely better
    readwhilewriting = common_cmd + [
        "--statistics=1",
        "--histogram=1",
        "--disable_auto_compactions=1",
        "--use-existing-db=true",
        f"--benchmark_write_rate_limit={opts.write_rate_limit}",
        "--db=/mnt/data",
        f"--duration={opts.duration}",
        # FIXME it seems that neard is using 4 viewclient
        f"--threads={NUM_IO_THREADS}",
        "--benchmarks=readwhilewriting,stats",
        "--compression_type=zstd",
    ]
    print("rocksdb readwhilewriting benchmark")
    run(readwhilewriting)


def die(msg: str) -> None:
    print(msg, file=sys.stderr)
    sys.exit(1)


def parse_args() -> DbBenchOptions:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db-path", help="Path to database directory", required=True)
    parser.add_argument(
        "--num-keys", help="Number of keys to insert into database", default=int(10e6)
    )
    parser.add_argument(
        "--duration", help="Benchmark duration in seconds", default=int(10)
    )
    # 9 mbyte/s is what we measured as top throughput in mainnet on 2022-10-07
    parser.add_argument(
        "--write-rate-limit",
        help="Number of byte/s to write while reading",
        default=int(9e6),
    )
    args = parser.parse_args()
    return DbBenchOptions(
        db_path=args.db_path,
        num_keys=args.num_keys,
        duration=args.duration,
        write_rate_limit=args.write_rate_limit,
    )


def increase_open_file_limit() -> None:
    soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
    if hard > soft:
        print(f"Increase filelimit to {hard}")
        resource.setrlimit(resource.RLIMIT_NOFILE, (hard, hard))


def main() -> None:
    opts = parse_args()
    if shutil.which("db_bench") is None:
        die("db_bench executable not found")
    increase_open_file_limit()
    db_bench(opts)


if __name__ == "__main__":
    main()
