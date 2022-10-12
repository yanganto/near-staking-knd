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

# =~ 170 GB, This is not in the same size as neard, but we also access the
# database uniform randomly rather than a zipfli distribution.
DB_KEY_NUM = int(5e7)

import os
import sys
import re
import subprocess
import argparse
import shutil
import resource
import json
from typing import IO, Any, Union, Callable, List, Dict, Optional, Text, NoReturn
from shlex import quote
from pathlib import Path
from enum import Enum
import io
from dataclasses import dataclass
import dataclasses

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
    report_path: Path
    db_path: Path
    num_keys: int
    write_rate_limit: int
    duration: int


def die(msg: str) -> NoReturn:
    print(msg, file=sys.stderr)
    sys.exit(1)


def parse_args() -> DbBenchOptions:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--report-path", help="Path to report file", default="report.json"
    )
    parser.add_argument("--db-path", help="Path to database directory", required=True)
    parser.add_argument(
        "--num-keys", help="Number of keys to insert into database", default=DB_KEY_NUM
    )
    parser.add_argument(
        "--duration", help="Benchmark duration in seconds", default=int(60)
    )
    # 9 mbyte/s is what we measured as top throughput in mainnet on 2022-10-07
    parser.add_argument(
        "--write-rate-limit",
        help="Number of byte/s to write while reading",
        default=int(9e6),
    )
    args = parser.parse_args()
    return DbBenchOptions(
        report_path=Path(args.report_path),
        # put rocksdb in subdirectory, so we can just delete it afterwards.
        db_path=Path(args.db_path) / "db",
        num_keys=args.num_keys,
        duration=args.duration,
        write_rate_limit=args.write_rate_limit,
    )


@dataclass
class DbBenchResult:
    micro_second_per_op: float
    ops_per_second: float
    duration: float
    operations: int
    bandwith_mb: float
    found: int
    searched: int
    raw_output: str


def parse_db_bench_output(profile_name: str, bench_output: str) -> DbBenchResult:
    for line in bench_output.splitlines():
        if line.startswith(f"{profile_name} :"):
            # >>> "readwhilewriting :       5.951 micros/op 1343424 ops/sec 60.090 seconds 80726992 operations;   73.6 MB/s (155288 of 10114999 found)".split()
            # ['readwhilewriting', ':', '5.951', 'micros/op', '1343424', 'ops/sec', '60.090', 'seconds', '80726992', 'operations;', '73.6', 'MB/s', '(155288', 'of', '10114999', 'found)']
            # [0                 , 1  , 2      , 3          , 4        , 5        , 6       , 7        , 8         , 9            , 10    , 11   ,  12       , 13  , 14        , 15      ]
            columns = line.split()
            assert (
                len(columns) == 16
            ), f"Rocksdb benchmark output line has not expected number of columns: expected: 16, got: {len(columns)}\n{line}"
            return DbBenchResult(
                micro_second_per_op=float(columns[2]),
                ops_per_second=float(columns[4]),
                duration=float(columns[6]),
                operations=int(columns[8]),
                bandwith_mb=float(columns[10]),
                found=int(columns[12].strip("(")),
                searched=int(columns[14]),
                raw_output=bench_output,
            )
    die(f"Did not find the benchmark results in db_bench output: {bench_output}\n")


def _db_bench(opts: DbBenchOptions) -> DbBenchResult:
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
        f"--num={opts.num_keys}",
        "--benchmarks=fillseqdeterministic",
    ]
    print(f"Initialize rocksdb at {opts.db_path} for benchmark")
    proc = run(seed_db, stdout=subprocess.PIPE)

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
    proc = run(readwhilewriting, stdout=subprocess.PIPE)
    return parse_db_bench_output("readwhilewriting", proc.stdout)


def db_bench(opts: DbBenchOptions) -> DbBenchResult:
    increase_open_file_limit()
    try:
        return _db_bench(opts)
    finally:
        shutil.rmtree(opts.db_path)


def increase_open_file_limit() -> None:
    soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
    if hard > soft:
        print(f"Increase filelimit to {hard}")
        resource.setrlimit(resource.RLIMIT_NOFILE, (hard, hard))


# for debugging
QUICK = False
FIO_RAMPUP = 10
FIO_RUNTIME = FIO_RAMPUP + 120
# Since we are using direct i/o here, we don't need to a super large file here.
# It just needs to be big enough to exceed the RAM / caches of our NVME drives.
FIO_SIZE = 100  # filesize in GB
if QUICK:
    FIO_RAMPUP = 2
    FIO_RUNTIME = FIO_RAMPUP + 8
    FIO_SIZE = 10


@dataclass
class FioResult:
    read_mean: float
    read_stddev: float
    # completion latency
    read_latency_mean: float
    read_latency_stddev: float
    write_mean: float
    write_stddev: float
    write_latency_mean: float
    write_latency_stddev: float


class Rw(Enum):
    r = 1
    w = 2
    rw = 3


def fio(
    path: Path,
    random: bool = False,
    rw: Rw = Rw.r,
    iops: bool = False,
) -> FioResult:
    """
    inspired by https://docs.oracle.com/en-us/iaas/Content/Block/References/samplefiocommandslinux.htm
    @param random: random vs sequential
    @param iops: return iops vs bandwidth
    @return (read_mean, stddev, write_mean, stdev) in kiB/s
    """
    cmd = []

    cmd += ["fio"]

    cmd += [f"--filename={path}/fio-file", f"--size={FIO_SIZE}GB", "--direct=1"]

    if rw == Rw.r and random:
        cmd += ["--rw=randread"]
    if rw == Rw.w and random:
        cmd += ["--rw=randwrite"]
    elif rw == Rw.rw and random:
        # fio/examples adds rwmixread=60 and rwmixwrite=40 here
        cmd += ["--rw=randrw"]
    elif rw == Rw.r and not random:
        cmd += ["--rw=read"]
    elif rw == Rw.w and not random:
        cmd += ["--rw=write"]
    elif rw == Rw.rw and not random:
        cmd += ["--rw=readwrite"]

    # Is io_uring the best?
    if iops:
        # 4k is also our key size in rocksdb
        # iodepth=32 to keep enough in-flight data
        # 2 CPUs is currently not enough to saturate NVMEs
        cmd += ["--bs=4k", "--ioengine=io_uring", "--iodepth=32", "--numjobs=4"]
    else:
        cmd += ["--bs=256k", "--ioengine=io_uring", "--iodepth=16", "--numjobs=4"]

    cmd += [
        f"--runtime={FIO_RUNTIME}",
        f"--ramp_time={FIO_RAMPUP}",
        "--time_based",
        "--group_reporting",
        "--name=generic_name",
        "--eta-newline=1",
    ]

    cmd += ["--output-format=json"]

    term = run(cmd, check=True, stdout=subprocess.PIPE)

    out = term.stdout
    j = json.loads(out)
    read = j["jobs"][0]["read"]
    write = j["jobs"][0]["write"]

    if iops:
        print(
            f"IOPS: read {read['iops_mean']:.2f}±{read['iops_stddev']:.2f}, "
            f"lat {read['clat_ns']['mean']/10e3:.2f}±{read['clat_ns']['stddev']/10e3:.2f}μs "
            "/ "
            f"write {write['iops_mean']:.2f}±{write['iops_stddev']:.2f}, "
            f"lat {write['clat_ns']['mean']/10e3:.2f}±{write['clat_ns']['stddev']/10e3:.2f}μs"
        )
        return FioResult(
            read["iops_mean"],
            read["iops_stddev"],
            read["clat_ns"]["mean"],
            read["clat_ns"]["stddev"],
            write["iops_mean"],
            write["iops_stddev"],
            read["clat_ns"]["mean"],
            read["clat_ns"]["stddev"],
        )
    else:
        print("Bandwidth read", float(read["bw_mean"]) / 1024 / 1024, "GB/s")
        print("Bandwidth write", float(write["bw_mean"]) / 1024 / 1024, "GB/s")
        return FioResult(
            read["bw_mean"],
            read["bw_dev"],
            read["clat_ns"]["mean"],
            read["clat_ns"]["stddev"],
            write["bw_mean"],
            write["bw_dev"],
            read["clat_ns"]["mean"],
            read["clat_ns"]["stddev"],
        )


# Stress-test system, we can compare the hash rate with https://xmrig.com/benchmark
def xmrig() -> float:
    regex = re.compile(
        r".*benchmark finished in (\d+\.\d+) seconds \((\d+\.\d+) h/s\).*"
    )
    with subprocess.Popen(
        ["xmrig", "--bench=5M", "--no-color"],
        stdout=subprocess.PIPE,
        text=True,
    ) as p:
        try:
            assert p.stdout is not None
            for line in p.stdout:
                print(line, end="")
                m = regex.match(line)
                if m is not None:
                    _ = float(m.group(1))
                    hash_rate = float(m.group(2))
                    return hash_rate
        finally:
            p.kill()
        print(
            "Could not get a benchmark result from xmrig. Check the logs above",
            file=sys.stderr,
        )
        sys.exit(1)


def inxi() -> str:
    p = run(
        ["inxi", "-F", "-a", "-i", "--slots", "-xxx", "-c0", "-Z", "-i", "-m"],
        stdout=subprocess.PIPE,
    )
    print(p.stdout)
    return p.stdout


def lstopo() -> str:
    # lstopo --if xml -i /tmp/foo.xml --of svg /tmp/foo.svg
    return run(
        ["lstopo", "--of", "xml"],
        stdout=subprocess.PIPE,
    ).stdout


def check_program(name: str) -> None:
    if shutil.which(name) is None:
        die(f"{name} executable not found")


def read_report(path: Path) -> dict[str, str]:
    stats: dict[str, str] = {}
    if not os.path.exists(path):
        return stats
    with open(path) as f:
        p = json.load(f)
        stats.update(p)
        return stats


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        return super().default(o)


def write_report(path: Path, stats: Dict[str, str]) -> None:
    path.parent.mkdir(exist_ok=True, parents=True)
    with open(path, "w") as f:
        json.dump(stats, f, indent=4, sort_keys=True, cls=EnhancedJSONEncoder)


class Report:
    def __init__(self, path: Path) -> None:
        self.path = path
        self.data = read_report(self.path)

    def __enter__(self) -> "Report":
        return self

    def run_benchmark(self, name: str, benchmark: Callable[[], Any]) -> None:
        if self.data.get(name):
            info(f"skip {name}")
            return

        info(f"### run {name} ###")

        self.data[name] = benchmark()
        write_report(self.path, self.data)

    def __exit__(self, exc_type: Any, exc_value: Any, traceback: Any) -> None:
        write_report(self.path, self.data)


def iperf():
    # "lon.speedtest.clouvider.net" "5200-5209" "Clouvider" "London, UK (10G)" "IPv4|IPv6" "ping.online.net" "5200-5209" "Online.net" "Paris, FR (10G)" "IPv4" "ping6.online.net" "5200-5209" "Online.net" "Paris, FR (10G)" "IPv6" "nyc.speedtest.clouvider.net" "5200-5209" "Clouvider" "NYC, NY, US (10G)" "IPv4|IPv6"  # pass
    pass


def main() -> None:
    opts = parse_args()
    check_program("db_bench")
    check_program("fio")
    check_program("inxi")
    check_program("xmrig")
    check_program("lstopo")
    check_program("inxi")

    if os.geteuid() != 0:
        die("This script needs to be executed as root")

    with Report(opts.report_path) as report:
        report.run_benchmark("inxi", inxi)
        report.run_benchmark("lstopo", lstopo)
        report.run_benchmark("xmrig", xmrig)
        report.run_benchmark(
            "fio-rand-read-only", lambda: fio(opts.db_path, True, Rw.r, True)
        )
        report.run_benchmark(
            "fio-rand-read-write", lambda: fio(opts.db_path, True, Rw.rw, True)
        )
        report.run_benchmark(
            "fio-seq-read-only", lambda: fio(opts.db_path, False, Rw.r, False)
        )
        report.run_benchmark(
            "fio-seq-read-write", lambda: fio(opts.db_path, False, Rw.rw, False)
        )
        report.run_benchmark("db_bench", lambda: db_bench(opts))


if __name__ == "__main__":
    main()
