#kuutamo-challenge

---

# Stake Wars: Episode III. Challenge xxx

### Status: **DRAFT**

 - Published on: 2022-07-xx
 - Updated on: 2022-xx-xx
 - Submitted by: kuutamo 
 - Rewards: xx
Setup a kuutamo High Availability NEAR Validator running on `shardnet`

The kuutamo HA NEAR Validator node distribution combines a Linux operating system (NixOS) preconfigured for security and performance for this use case, kuutamod, consuld and neard.

kuutamod is a distributed supervisor for neard that implements failover. To avoid having two active validators running simultaneously, kuutamod uses consul by acquiring a distributed lock.

For support join [kuutamo-chat on Matrix](https://matrix.to/#/#kuutamo-chat:kuutamo.chat) 

## Tasks:

 1. Deploy kuutamod on localnet following [this guide](https://github.com/kuutamolabs/kuutamod/blob/main/docs/run.md#running-a-localnet-cluster-for-testing-and-development). Write a blog post documenting your experience.
 2. Deploy a HA pool using kuutamo on with a name appended with `_kuutamo` on shardnet (i.e. `mypoolname_kuutamo.factory.shardnet.near`) following [this guide](https://github.com/kuutamolabs/kuutamod/blob/main/docs/run.md#running-on-mainnet-testnet-or-shardnet). Write a blog post documenting your experience.

## Deliverables

 - Blog for localnet deployment
 - Blog for shardnet deployment. On each kuutamo node, once your system is operational, run the commands below and include in blog.
```console
$ nixos-version
$ journalctl -u kuutamod.service | grep 'state changed'
$ systemctl status kuutamod
```

## Useful links:

[kuutamo NEAR Validator GitHub/Docs](https://github.com/kuutamolabs/kuutamod)

[Installing NixOS](https://nixos.org/manual/nixos/stable/index.html#ch-installation)

[An opinionated guide for developers getting things done using the Nix ecosystem](https://nix.dev/)

[Nix to Debian phrasebook](https://nixos.wiki/wiki/Nix_to_Debian_phrasebook)
