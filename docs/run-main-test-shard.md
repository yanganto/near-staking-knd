# Running on `mainnet` or `testnet`

## Single node kneard

This part of the tutorial assumes that you have installed a computer on which.
[NixOS](https://nixos.org/manual/nixos/stable/#sec-installation).
This is not yet a failover setup, as we will only use a single machine for simplicity.
How to convert this setup into a cluster setup is described in the next section.
To use the NixOS modules we provide in this repository, you also need to enable Flakes in NixOS.
To do this, add these lines to your `configuration.nix`...

```nix
{
  nix.extraOptions = ''
    experimental-features = nix-command flakes
  '';
}
```

and create a `flake.nix` file in `/etc/nixos/` [More info on flakes](https://nixos.wiki/wiki/Flakes#Using_nix_flakes_with_NixOS).

In your `flake.nix` you have to add the `kneard` flake as source and import
the nixos modules from it into your configuration.nix.

```nix
{
  inputs = {
    # This is probably already there.
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";

    # This is the line you need to add.
    near-staking-knd.url = "github:kuutamolabs/near-staking-knd";
  };
  outputs = { self, nixpkgs, near-staking-knd }: {
    # Replace 'my-validator' with your hostname here.
    nixosConfigurations.my-validator = nixpkgs.lib.nixosSystem {
      # Our neard package is currently only tested on x86_64-linux.
      system = "x86_64-linux";
      modules = [
        ./configuration.nix

        # Optional: This adds a our binary cache so you don't have to compile neard/kneard yourself.
        # The binary cache module, won't be effective on the first run of nixos-rebuild, but you can specify it also via command line like this:
        # $ nixos-rebuild switch --option  extra-binary-caches "https://cache.garnix.io" --option extra-trusted-public-keys "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
        near-staking-knd.nixosModules.kuutamo-binary-cache

        # These are the modules provided by our flake
        near-staking-knd.nixosModules.neard-testnet
        # or if you want to join other networks, use one of these as needed.
        # near-staking-knd.nixosModules.neard-mainnet
        near-staking-knd.nixosModules.kneard
      ];
    };
  };
}
```

## Bootstrap from S3

To bootstrap neard quickly, you can use an S3 backup of the chain database.

### mainnet / testnet
For `mainnet` and `testnet`, these are provided in the `near-protocol-public`
S3 bucket.

You need to determine the latest timestamp manually, and configure
the config with the URL:

```
$ nix-shell -p awscli --command 'aws s3 --no-sign-request cp s3://near-protocol-public/backups/testnet/rpc/latest -'
2022-07-15T11:00:30Z
```

In this case, the full s3 backup URL (to be used in the config below, as
`kuutamo.neard.s3.dataBackupDirectory`) is
`s3://near-protocol-public/backups/testnet/rpc/2022-07-15T11:00:30Z`.

For `mainnet` replace the word `testnet` in the urls above.

---

Create a new file called `kneard.nix` next to your `configuration.nix` in `/etc/nixos/`.
If your NixOS configuration is managed via a git repository, do not forget to run `git add kneard.nix`.

Add the following configuration to the `/etc/nixos/kneard.nix` file:

```nix
{
  # Consul wants to bind to a network interface. You can get your interface as follows:
  # $ ip route get 8.8.8.8
  # 8.8.8.8 via 131.159.102.254 dev enp24s0f0 src 131.159.102.16 uid 1000
  #   cache
  # This becomes relevant when you scale up to multiple machines.
  services.consul.interface.bind = "enp24s0f0";
  services.consul.extraConfig.bootstrap_expect = 1;

  # This is the URL we calculated above.
  kuutamo.neard.s3.dataBackupDirectory = "s3://near-protocol-public/backups/testnet/rpc/2022-07-15T11:00:30Z";
  # kuutamo.neard.s3.dataBackupDirectory = "s3://near-protocol-public/backups/mainnet/rpc/2022-07-15T11:00:31Z";

  # We create these keys after the first 'nixos-rebuild switch'
  # As these files are critical, we also recommend tools like https://github.com/Mic92/sops-nix or https://github.com/ryantm/agenix
  # to securely encrypt and manage these files. For both sops-nix and agenix, set the owner to 'neard' so that the service can read it.
  kuutamo.kneard.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kneard.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
}
```

Import this file in your `configuration.nix`:

```nix
{
  imports = [ ./kneard.nix ];
}
```

Before we can move on generating validator keys, we need first create the neard user.

```
nixos-rebuild switch --flake /etc/nixos#my-validator
```

The first switch will take longer since it blocks on downloading the s3 data backup (around 300GB).
You can follow the progress by running: `sudo journalctl -u kneard -f`.

#### Node keys / generating the active validator key

Note that with kneard there will be one validator and node key for the active
validator, while each validator also has its own non-validator node key, which
is used during passive mode. The passive keys are created automatically by
kneard.

The next step is to generate and install the active validator key and validator
node key.



Run the following command but replace
`kuutamo-test_kuutamo.poolv1.near`, with your own pool id, and delete as appropriate where you see <mainnet|testnet>

```console
$ export NEAR_ENV=<mainnet|testnet>
$ nix run github:kuutamoaps/near-staking-knd#near-cli generate-key kuutamo-test_kuutamo.poolv1.near
$ nix run github:kuutamoaps/near-staking-knd#near-cli generate-key node_key
```

You then must edit these files and change `private_key` to `secret_key`.

```console
$ nano ~/.near-credentials/<mainnet|testnet>/kuutamo-test_kuutamo.poolv1.near.json
$ nano ~/.near-credentials/<mainnet|testnet>/node_key.json
```

You can then install them like this (but replace
`kuutamo-test_kuutamo.poolv1.near`, with your own pool id, and delete as appropriate where you see <mainnet|testnet>):

```console
$ sudo install -o neard -g neard -D -m400 ~/.near-credentials/<mainnet|testnet>/kuutamo-test_kuutamo.poolv1.near.json /var/lib/secrets/validator_key.json
$ sudo install -o neard -g neard -D -m400 ~/.near-credentials/<mainnet|testnet>/node_key.json /var/lib/secrets/node_key.json
```

You will now need to run `systemctl restart kuutamod` so that it picks up the keys. If everything
went well, you should be able to reach kneard's prometheus exporter url:

```
$ curl http://localhost:2233/metrics
# HELP kuutamod_state In what state our supervisor statemachine is
# TYPE kuutamod_state gauge
kuutamod_state{type="Registering"} 0
kuutamod_state{type="Shutdown"} 0
kuutamod_state{type="Startup"} 0
kuutamod_state{type="Syncing"} 1
kuutamod_state{type="Validating"} 0
kuutamod_state{type="Voting"} 0
# HELP kuutamod_uptime Time in milliseconds how long daemon is running
# TYPE kuutamod_uptime gauge
kuutamod_uptime 1273658
```

Once neard is synced with the network, you should see a kneard listed as an active validator using `kneard-ctl`:

```
$ kneard-ctl active-validator
Name: river
```

where `name` is the kuutamo node id.

## Multi-Node kuutamo cluster

Once your single-node kneard setup works, you can scale out to multiple nodes by changing your `kneard.nix`
like this:

```
{

  # Same as above, this needs to be an interface should be used to connect to your other machines
  # If you've come from the AWS testnet guide, note you may need to change this.
  services.consul.interface.bind = "enp24s0f0";

  # this now needs to be increased to the number of consul nodes your are adding
  services.consul.extraConfig.bootstrap_expect = 3;

  # We allow these ports for our consul server. Here we assume a trusted network. If this is not the case, read about
  # setting up encryption and authentication for consul: https://www.consul.io/docs/security/encryption
  networking.firewall = {
    allowedTCPPorts = [
      8301 # lan serf
      8302 # wan serf
      8600 # dns
      8500 # http api
      8300 # RPC address
    ];
    allowedUDPPorts = [
      8301 # lan serf
      8302 # wan serf
      8600 # dns
    ];
  };

  # add here the ip addresses or domain names of other hosts, that you want to add to the cluster
  services.consul.extraConfig.retry_join = [
    "node0.mydomain.tld"
    "node1.mydomain.tld"
    "node3.mydomain.tld"
  ];

  # Everything below stays the same.

  # This is the URL we calculated above:
  kuutamo.neard.s3.dataBackupDirectory = "s3://near-protocol-public/backups/testnet/rpc/2022-07-13T11:00:40Z";

  # We create these keys after the first 'nixos-rebuild switch'
  # As these files are critical, we also recommend tools like https://github.com/Mic92/sops-nix or https://github.com/ryantm/agenix
  # to securely encrypt and manage these files. For both sops-nix and agenix, set the owner to 'neard' so that the service can read it.
  kuutamo.kneard.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kneard.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
}
```

Do not forget to also copy `/var/lib/secrets/validator_key.json` and `/var/lib/secrets/node_key.json` from your first machine to the other nodes.
After running `nixos-rebuild switch` on each of them.
Check that your consul cluster is working:

If you access `http://localhost:8500/v1/status/peers` from any of the hosts, it should contain all node ips of your consul cluster:

```
curl http://localhost:8500/v1/status/peers
["131.0.0.1:8300","131.0.0.2:8300","131.0.0.3:8300"]
```

Furthermore `http://localhost:8500/v1/status/leader` should contain the consul cluster leader:

```
curl http://localhost:8500/v1/status/leader
"131.159.102.16:8300"
```

Just like in the `localnet` example, you can query
`http://localhost:2233/metrics` on each host or use `kneard-ctl` to see which
host is currently the designated validator.

# Further reading

- [Configuration](./configuration.md): All configuration options in kneard
- [Reset neard](./reset-neard): How to reset neard, i.e. after a network fork or when changing the chain.
- [Failover algorithm](./failover-algorithm.md) describes the runtime behavior of kneard in depth
