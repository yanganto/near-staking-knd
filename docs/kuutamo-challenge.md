#kuutamo-challenge

---

# Stake Wars: Episode III. Challenge xxx

### Status: **DRAFT**

 - Published on: 2022-07-xx
 - Updated on: 2022-xx-xx
 - Submitted by: kuutamo 
 - Rewards: xx
Setup a kuutamo High Availability NEAR Validator running on `shardnet`

The kuutamo NEAR Validator combines a preconfigured security and performance Linux operating system (NixOS), kuutamod, consul and neard.

kuutamod is a distributed supervisor for neard that implements failover. To avoid having two active validators running simultaneously, kuutamod uses consul by acquiring a distributed lock.

For support join [kuutamo-chat on Matrix](https://matrix.to/#/#kuutamo-chat:kuutamo.chat)

## Tasks:

 1. Deploy kuutamod on localnet following 
 2. Deploy a HA pool on shardnet as your_poolname_kuutamo.factory.near. Write a blog post documenting your experience. (3/5 of challenge rewards)

## Deliverables

 - Blog for localnet deployment
 - Blog for shardnet deployment
 - On each kuutamo node, run the commands below and screenshot outputs in blogs.
```
$ nixos-version
$ journalctl -u kuutamod.service | grep 'state changed'
$ systemctl status kuutamod
```

## Useful links:

[kuutamo NEAR Validator GitHub/Docs](https://github.com/kuutamolabs/kuutamod)

[Installing NixOS](https://nixos.org/manual/nixos/stable/index.html#ch-installation)

[An opinionated guide for developers getting things done using the Nix ecosystem](https://nix.dev/)

[Nix to Debian phrasebook](https://nixos.wiki/wiki/Nix_to_Debian_phrasebook)