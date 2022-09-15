# NEAR staking knd (kuutamo node distribution)

This project provides a distributed supervisor for neard that implements failover for
[NEAR validators](https://near.org/validators/). NEAR is an application
platform built on top of the NEAR protocol blockchain. Validator nodes, run by
the community, provide computational resources to the NEAR network and collect
monetary rewards at regular intervals, based on the volume of work (blocks and chunks
produced). Validators who do not complete the work assigned to them receive
fewer rewards and may be excluded from the group of validators who are allowed
to validate for some time.

kuutamod allows multiple NEAR validators to operate in an active-passive setup.

One validator becomes the active validator node, and is started with the
validator keys, while the other nodes stay synchronised with the blockchain
ready to take over if needed.

In the event of a failure, i.e. a neard crash, network split or hardware
failure of the active validator node, a passive instance will get be promoted
to an active validator. This is accomplished by kuutamod automatically
restarting a passive node with validator keys. Future work is planned to be
able to switch active/passive mode at runtime.

To avoid having two active validators running at the same time, kuutamod uses
[consul](https://www.consul.io/) by acquiring a distributed lock.

## Status: beta

kuutamod is continously tested and also used in production for validators.
However `kuutamod`'s interface is still under active development and might
change significantly.

## Docs

- Tutorials:
  - [Build and install from source](docs/build.md)
  - [Runon a localnet](docs/run-localnet.md)
  - [Run on testnet / mainnet](docs/run-main-test-shard.md)
  - [Quickstart Lab - testnet on AWS](docs/testnet-on-aws-lab.md)
  <!-- TODO - [Monitoring](docs/monitoring.md) -->
- References:
  - [Configuration](docs/configuration.md)
- [Architecture](docs/architecture.md)
  - [Failover algorithm](docs/failover-algorithm.md)

---
kuutamolabs  
[GitHub](https://github.com/kuutamolabs/near-staking-knd) | [Matrix](https://matrix.to/#/#kuutamo-chat:kuutamo.chat)
