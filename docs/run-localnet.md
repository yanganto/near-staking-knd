# Run a kuutamod failover setup

This tutorial is split into two parts. First we show how to run kuutamod
locally in [localnet](https://docs.near.org/docs/concepts/networks#localnet).
This allows you to understand how kuutamod works and play around with
failover kuutamod and neard.

**NB: In production, Systemd or another supervisor must be used as this will terminate 
neard in the case kuutamod crashes.**

The second part shows how to deploy validator ha setup for
[testnet](https://docs.near.org/docs/concepts/networks#testnet) using
[NixOS](https://nixos.org/).

If you want to get an high-level overview of how kuutamod works, you can also
read the [Architecture](./architecture.md) page.

## Running a `localnet` cluster for testing and development

This is the easiest way to test kuutamod, which requires the least amount of
resources as it does not require downloading large amounts of chain data. We
will set up a local NEAR network and allow kuutamod to connect to it. This
tutorial is intended to work on a single computer without the need for an open
port to the internet. This setup is **not** recommended for production use.
In particular a supervisor like systemd is recommended for running kuutamod to
stop neard in case kuutamod is killed.

### Requirements

Install the nix package manager (as described [here](https://nix.dev/tutorials/install-nix)),
and you will get all dependencies needed by running `nix develop` from the source directory
of kuutamod

```console
$ git clone https://github.com/kuutamolabs/near-staking-knd
$ nix --extra-experimental-features "nix-command flakes" develop
```

If you don't use nix you will need the following executables in your `$PATH`.

- [consul](https://www.consul.io/): This provides a distributed lock for
  kuutamod to detect liveness and prevent two validators from running at the
  same time.

- [neard](https://github.com/near/nearcore/releases/latest): Kuutamod will run this binary.

- [hivemind](https://github.com/DarthSim/hivemind): This is optionally required
  to run execute our [Procfile](../Procfile). You can also manually execute the
  commands contained in this file.

- [Python](https://www.python.org/) for some of the setup scripts.


After installing the dependencies or running `nix develop`, run the command hivemind:

```console
$ hivemind
```

Hivemind starts consul, sets up the localnet configuration and starts two neard
instances for this network. It should be noted that only one consul server is
running. In a production setup, one should run a
[cluster](https://www.consul.io/docs/install/bootstrapping), otherwise this
consul server becomes a single point of failure. The scripts also set up keys
and configuration for two kuutamod instances. The localnet configuration for all
nodes is stored in `.data/near/localnet`.

Note: If you built kuutamod from source using `cargo build`, the binary is in
`target/debug` or `target/release`, depending on whether you have a debug or
release build.

Next, start kuutamod in a **new terminal window** so you can run commands whilst hivemind is running. You will
also need to `cd kuutamod` and run `nix --extra-experimental-features "nix-command flakes" develop`, if using nix,
to get the dependencies in your `$PATH` in this new session. Then:

```console
$ cargo build
$ ./target/debug/kuutamod --neard-home .data/near/localnet/kuutamod0/ \
  --voter-node-key .data/near/localnet/kuutamod0/voter_node_key.json \
  --validator-node-key .data/near/localnet/node3/node_key.json \
  --validator-key .data/near/localnet/node3/validator_key.json \
  --near-boot-nodes $(jq -r .public_key < .data/near/localnet/node0/node_key.json)@127.0.0.1:33301
```

You can check if it becomes a validator by running the command `curl`.

```console
$ curl http://localhost:2233/metrics
# HELP kuutamod_neard_restarts How often neard has been restarted
# TYPE kuutamod_neard_restarts counter
kuutamod_neard_restarts 1
# HELP kuutamod_state In what state our supervisor statemachine is
# TYPE kuutamod_state gauge
kuutamod_state{type="Registering"} 0
kuutamod_state{type="Shutdown"} 0
kuutamod_state{type="Startup"} 0
kuutamod_state{type="Syncing"} 0
kuutamod_state{type="Validating"} 1
kuutamod_state{type="Voting"} 0
# HELP kuutamod_uptime Time in milliseconds how long daemon is running
# TYPE kuutamod_uptime gauge
kuutamod_uptime 81917
```

This retrieves data from the [prometheus](https://prometheus.io/) monitoring endpoint of kuutamod.

The line `kuutamod_state{type="Validating"} 1` indicates that `kuutamod` has set
up neard as a validator, as you can also see from the neard home directory:

```console
$ ls -la .data/near/localnet/kuutamod0/
.rw-r--r-- 2,3k joerg 12 Jul 14:12 config.json
drwxr-xr-x    - joerg 12 Jul 14:12 data/
.rw-r--r-- 6,7k joerg 12 Jul 13:47 genesis.json
lrwxrwxrwx   73 joerg 12 Jul 14:12 node_key.json -> /home/joerg/work/kuutamo/kuutamod/.data/near/localnet/node3/node_key.json
lrwxrwxrwx   78 joerg 12 Jul 14:12 validator_key.json -> /home/joerg/work/kuutamo/kuutamod/.data/near/localnet/node3/validator_key.json
.rw-------  214 joerg 12 Jul 13:47 voter_node_key.json
```

The validator key has been symlinked and the node key has been replaced with the
node key specified in `-validator-node-key`.

After that you can also start a second `kuutamod` instance as follows:

```console
$ ./target/debug/kuutamod \
  --exporter-address 127.0.0.1:2234 \
  --validator-network-addr 0.0.0.0:24569 \
  --voter-network-addr 0.0.0.0:24570 \
  --neard-home .data/near/localnet/kuutamod1/ \
  --voter-node-key .data/near/localnet/kuutamod1/voter_node_key.json \
  --validator-node-key .data/near/localnet/node3/node_key.json \
  --validator-key .data/near/localnet/node3/validator_key.json \
  --near-boot-nodes $(jq -r .public_key < .data/near/localnet/node0/node_key.json)@127.0.0.1:33301
```

Note that we choose different network ports to not collide with the first
kuutamod instance on the same machine. Also, we choose a separate directory
while using the same keys for `--voter-node-key` and `--validator-node-key`.
The second kuutamod has its metrics endpoint at `http://localhost:2234/metrics`.
Again, with `curl`, we can see that it has entered the Voting state, as there is
already another kuutamod instance registered:

```
$ curl http://localhost:2234/metrics
# HELP kuutamod_state In what state our supervisor statemachine is
# TYPE kuutamod_state gauge
kuutamod_state{type="Registering"} 0
kuutamod_state{type="Shutdown"} 0
kuutamod_state{type="Startup"} 0
kuutamod_state{type="Syncing"} 0
kuutamod_state{type="Validating"} 0
kuutamod_state{type="Voting"} 1
# HELP kuutamod_uptime Time in milliseconds how long daemon is running
# TYPE kuutamod_uptime gauge
kuutamod_uptime 10412
```

If we look at its neard home directory we can also see that no validator key is
present and the node key specified by `--voter-node-key` is symlinked:

```
$ ls -la .data/near/localnet/kuutamod1
.rw-r--r-- 2,3k joerg 12 Jul 14:20 config.json
drwxr-xr-x    - joerg 12 Jul 14:20 data/
.rw-r--r-- 6,7k joerg 12 Jul 13:47 genesis.json
lrwxrwxrwx   83 joerg 12 Jul 14:20 node_key.json -> /home/joerg/work/kuutamo/kuutamod/.data/near/localnet/kuutamod1/voter_node_key.json
.rw-------  214 joerg 12 Jul 13:47 voter_node_key.json
```

If we now stop the first `kuutamod` instance by pressing `ctrl-c`...

```
2022-07-12T14:38:22.810412Z  WARN neard: SIGINT, stopping... this may take a few minutes.
level=info pid=2119211 message="SIGINT received" target="kuutamod::exit_signal_handler" node_id=node
level=info pid=2119211 message="state changed: Voting -> Shutdown" target="kuutamod::supervisor" node_id=node
level=warn pid=2119211 message="Termination timeout reached. Send SIGKILL to neard!" target="kuutamod::proc" node_id=node
```

... we can see that the second instance takes over:

```
2022-07-12T14:52:02.827213Z  INFO stats: #       0 CyjBSLQPeET76Z2tZP2otY8gDFsxANBgobf57o9Mzi8e 4 validators 0 peers ⬇ 0 B/s ⬆ 0 B/s 0.00 bps 0 gas/s CPU: 0%, Mem: 34.0 MB
level=info pid=2158051 message="state changed: Voting -> Validating" target="kuutamod::supervisor" node_id=node
2022-07-12T14:52:04.271448Z  WARN neard: SIGTERM, stopping... this may take a few minutes.
2022-07-12T14:52:09.281715Z  INFO neard: Waiting for RocksDB to gracefully shutdown
2022-07-12T14:52:09.281725Z  INFO db: Waiting for the 1 remaining RocksDB instances to gracefully shutdown
2022-07-12T14:52:09.281746Z  INFO db: Dropped a RocksDB instance. num_instances=0
2022-07-12T14:52:09.281772Z  INFO db: All RocksDB instances performed a graceful shutdown
level=warn pid=2158051 message="Cannot reach neard status api: Failed to get status" target="kuutamod::supervisor" node_id=node
2022-07-12T14:52:09.295345Z  INFO neard: version="1.27.0" build="nix:1.27.0" latest_protocol=54
2022-07-12T14:52:09.295956Z  INFO near: Opening store database at ".data/near/localnet/kuutamod1/data"
2022-07-12T14:52:09.312159Z  INFO db: Created a new RocksDB instance. num_instances=1
2022-07-12T14:52:09.312801Z  INFO db: Dropped a RocksDB instance. num_instances=0
2022-07-12T14:52:09.401450Z  INFO db: Created a new RocksDB instance. num_instances=1
2022-07-12T14:52:09.440197Z  INFO near_network::peer_manager::peer_manager_actor: Bandwidth stats total_bandwidth_used_by_all_peers=0 total_msg_received_count=0 max_max_record_num_messages_in_progress=0
2022-07-12T14:52:09.454305Z  INFO stats: #       0 CyjBSLQPeET76Z2tZP2otY8gDFsxANBgobf57o9Mzi8e Validator | 4 validators 0 peers ⬇ 0 B/s ⬆ 0 B/s NaN bps 0 gas/s
2022-07-12T14:52:19.457739Z  INFO stats: #       0 CyjBSLQPeET76Z2tZP2otY8gDFsxANBgobf57o9Mzi8e Validator | 4 validators 0 peers ⬇ 0 B/s ⬆ 0 B/s 0.00 bps 0 gas/s CPU: 1%, Mem: 34.7 MB
```

This currently requires a restart of `neard` so that it loads the `validator node key`.

```
$ curl http://localhost:2234/metrics
# HELP kuutamod_neard_restarts How often neard has been restarted
# TYPE kuutamod_neard_restarts counter
kuutamod_neard_restarts 1
# HELP kuutamod_state In what state our supervisor statemachine is
# TYPE kuutamod_state gauge
kuutamod_state{type="Registering"} 0
kuutamod_state{type="Shutdown"} 0
kuutamod_state{type="Startup"} 0
kuutamod_state{type="Syncing"} 0
kuutamod_state{type="Validating"} 1
kuutamod_state{type="Voting"} 0
# HELP kuutamod_uptime Time in milliseconds how long daemon is running
# TYPE kuutamod_uptime gauge
kuutamod_uptime 43610
```

```
$ ls -la .data/near/localnet/kuutamod1
.rw-r--r-- 2,3k joerg 12 Jul 14:54 config.json
drwxr-xr-x    - joerg 12 Jul 14:54 data/
.rw-r--r-- 6,7k joerg 12 Jul 14:54 genesis.json
lrwxrwxrwx   73 joerg 12 Jul 14:54 node_key.json -> /home/joerg/work/kuutamo/kuutamod/.data/near/localnet/node3/node_key.json
lrwxrwxrwx   78 joerg 12 Jul 14:54 validator_key.json -> /home/joerg/work/kuutamo/kuutamod/.data/near/localnet/node3/validator_key.json
.rw-------  214 joerg 12 Jul 14:54 voter_node_key.json
```

# Further reading

See the next [chapter](./run-main-test-shard.md) to learn how to run kuutamod
cluster in production using our NixOS module.
