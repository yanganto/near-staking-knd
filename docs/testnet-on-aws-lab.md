# Lab: Single node kuutamo near validator up on testnet using AWS.

- Get [NixOS EC2 AMI](https://nixos.org/download.html#nixos-amazon)
  In this demo I used N.Virginia (us-east-1): `ami-0223db08811f6fb2d` NB: Each region uses an AMI with a different name so double check that you picked the correct region on the NixOS site if the AMI doesn't show up in the AWS UI.
  
  ![image](https://user-images.githubusercontent.com/38218340/185245850-28b37993-3645-491a-b6fd-bb908737bf8d.png)
  
- Setup VM
  AWS > EC2 > AMIs > `ami-0223db08811f6fb2d` > Launch instance from AMI (we tested on c6a.4xlarge with 500GB gp3 disk) > Launch instance
- SSH to instance

#### Edit `configuration.nix` so it is as below: `nano /etc/nixos/configuration.nix`
```nix
{ modulesPath, ... }: {
  imports = [ "${modulesPath}/virtualisation/amazon-image.nix" ./kuutamod.nix];
  ec2.hvm = true;

  nix.extraOptions = ''
  experimental-features = nix-command flakes
  '';
  
  swapDevices = [{
    device = "/swapfile";
    size = 4096;
  }];
}
```

#### Add `flake.nix` file as below: `nano /etc/nixos/flake.nix`
```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
    kuutamod.url = "github:kuutamolabs/kuutamod";
  };
  outputs = { self, nixpkgs, kuutamod }: {
    nixosConfigurations.validator = nixpkgs.lib.nixosSystem {
      # Our neard package is currently only tested on x86_64-linux.
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        
        # Optional: This adds a our binary cache so you don't have to compile neard/kuutamod yourself.
        # The binary cache module, won't be effective on the first run of nixos-rebuild, but you can specify it also via command line like this:
        # $ nixos-rebuild switch --option  extra-binary-caches "https://cache.garnix.io" --option extra-trusted-public-keys "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=" --flake /etc/nixos#validator
        self.inputs.kuutamod.nixosModules.kuutamo-binary-cache

        kuutamod.nixosModules.neard-testnet
        kuutamod.nixosModules.kuutamod
      ];
    };
  };
}
```

#### Bootstrap from S3

To bootstrap neard quickly, you can use an S3 backup of the chain database.
For `mainnet` and `testnet`, these are provided in the `near-protocol-public`
S3 bucket.

You need to determine the latest timestamp manually, and configure
the config with the URL:

```
$ nix --extra-experimental-features "nix-command flakes" shell nixpkgs#awscli2 -c aws s3 --no-sign-request cp s3://near-protocol-public/backups/testnet/rpc/latest -
2022-08-23T11:00:30Z
```

In this case, the full s3 backup URL (to be used in the config below, as
`kuutamo.neard.s3.dataBackupDirectory`) is
`s3://near-protocol-public/backups/testnet/rpc/2022-08-23T11:00:30Z`


#### Add `kuutamod.nix` file as below: `nano /etc/nixos/kuutamod.nix`
```nix
{
  # consul is here because you can add more kuutamod nodes later and create an Active/Passive HA cluster.
  # Consul wants to bind to a network interface. You can get your interface as follows:
  # $ ip route get 8.8.8.8
  # 8.8.8.8 via 131.159.102.254 dev enp24s0f0 src 131.159.102.16 uid 1000
  #   cache
  # This becomes relevant when you scale up to multiple machines.
  services.consul.interface.bind = "ens5";
  services.consul.extraConfig.bootstrap_expect = 1;
  
  # This is the URL we calculated above:
  kuutamo.neard.s3.dataBackupDirectory = "s3://near-protocol-public/backups/testnet/rpc/2022-07-15T11:00:30Z";

  kuutamo.kuutamod.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kuutamod.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";
}
```

#### Rebuild and switch to new configuration
If you are wanting to use the binary cache:

```console
$ nixos-rebuild boot --option  extra-binary-caches "https://cache.garnix.io" --option extra-trusted-public-keys "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=" --flake /etc/nixos#validator
```
If not, and you want compile neard and kuutamod on the machine (remember to comment out this line in `flake.nix`  `self.inputs.kuutamod.nixosModules.kuutamo-binary-cache`):

```console
$ nixos-rebuild boot --flake /etc/nixos#validator
warning: creating lock file '/etc/nixos/flake.lock'
building the system configuration...
updating GRUB 2 menu...
```

#### Reboot the machine
```console
$ reboot
```

SSH back into the machine. It can take a couple of minutes to be ready. Then run `journalctl -u kuutamod.service -n 10` 
This will show you the 10 most recent kuutamod logs. If you logged straight back in after reboot you'll probably see something like this:
![image](https://user-images.githubusercontent.com/38218340/186202697-8b83218a-188d-4610-8ecd-c6025ca9bf89.png)

This is the S3 backup download, and at time of writing takes about an hour to download.

You can continue the rest of this setup but note this needs to complete first, then the block/chunk validations, before you'll move on from the 'Syncing' kuutamod state.

#### Create keys

Note that with kuutamod there will be one validator and node key for the active
validator, while each validator also has its own non-validator node key, which
is used during passive mode. The passive keys are created automatically by
kuutamod.

The next step is to generate and install the active validator key and validator
node key.

Run the following command but replace
`kuutamo-test_kuutamo.pool.f863973.m0`, with your own pool id

```console
$ export NEAR_ENV=testnet
$ nix run github:kuutamoaps/kuutamod#near-cli generate-key kuutamo-test_kuutamo.pool.f863973.m0
$ nix run github:kuutamoaps/kuutamod#near-cli generate-key node_key
```

You then must edit these files and change `private_key` to `secret_key`.

```console
$ sed -i -e 's/private_key/secret_key/' ~/.near-credentials/testnet/kuutamo-test_kuutamo.f863973.m0.json ~/.near-credentials/testnet/node_key.json
```

You can then install them like this (but replace
`kuutamo-test_kuutamo.f863973.m0`, with your own pool id:

```console
$ sudo install -o neard -g neard -D -m400 ~/.near-credentials/testnet/kuutamo-test_kuutamo.f863973.m0.json /var/lib/secrets/validator_key.json
$ sudo install -o neard -g neard -D -m400 ~/.near-credentials/testnet/node_key.json /var/lib/secrets/node_key.json
```

You will then need restart kuutamod with `systemctl restart kuutamod` so that it picks up the key. If everything
went well, you should be able to reach kuutamod's prometheus exporter url:

```console
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
kuutamod_uptime 3447978
```

Once neard is synced with the network, you should see a kuutamod listed as an active validator using `kuutamoctl`:
```console
$ kuutamoctl active-validator
Name: river
```
where `Name` is the kuutamo node id.

You can view logs in the systemd journal
```console
$ journalctl -u kuutamod.service -f
Jul 17 21:43:50 river kuutamod[44389]: 2022-07-17T21:43:50.898176Z  INFO stats: # 1102053 7zgkxdDiKBoqud9DuSC47cwZ94e63BwGj1NNKs93JcLs Validator | 100 validators 29 peers ⬇ 345 kB/s ⬆ 485 kB/s 0.80 bps 0 gas/s CPU: 0%, Mem: 1.77 GB
```

---
#### Next Steps

- You can add more nodes to create an active/passive ha cluster. See [this section](https://github.com/kuutamolabs/kuutamod/blob/main/docs/run-main-test-shard.md#multi-node-kuutamo-cluster) for more information. 

---
kuutamolabs  
[GitHub](https://github.com/kuutamolabs/kuutamod) | [Matrix](https://matrix.to/#/#kuutamo-chat:kuutamo.chat)
