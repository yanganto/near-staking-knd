Our mission is to enable everyone to deploy and run resilient, secure and performant protocol infrastructure.

In the world of software, you usually need to decide between using a managed SaaS or running everything yourself in a self-hosted environment, which means handling all the operations to keep things running smoothly. At kuutamo we believe that there is a better way. A hybrid cloud first way. A next generation cloud. Our packaged services can be deployed anywhere, to any cloud, bare metal, and to our users own infrastructure. We aim to provide all the updates, monitoring and ops tooling needed, along with world-class SRE for protocol and infrastructure support services.

# NEAR Staking Node using kneard from kuutamo

## Prerequisites

- Server/node: Any Linux OS
- Workstation/development machine: Any Linux OS

These are two different machines. The kneard manager, `kneard-mgr` will run on your workstation. It will talk over SSH to your server/node. During install the server/node will be wiped and a fresh kuutamo near distribution will be installed onto it.


## Server Setup

You will need a server with any Linux OS installed. You will need SSH root access with a key. 

We have validated:

- [OVH](https://www.ovhcloud.com/en-gb/bare-metal/advance/adv-1/) - Advance 1 Gen 2, 64GB RAM, 2 x 960GB NVMe, with Ubuntu

Before [installing Ubuntu on the server](https://support.us.ovhcloud.com/hc/en-us/articles/115001775950-How-to-Install-an-OS-on-a-Dedicated-Server), [add your workstation SSH key](https://docs.ovh.com/gb/en/dedicated/creating-ssh-keys-dedicated/#importing-your-ssh-key-into-the-ovhcloud-control-panel_1).

- [Latitude](https://www.latitude.sh/features) - c3.medium.x86, with Ubuntu

Before [installing Ubuntu on the server](https://docs.latitude.sh/docs/deployments-and-reinstalls#deploying-a-server), [add your workstation SSH key](https://docs.latitude.sh/docs/ssh#adding-your-ssh-key).


## Workstation Setup

1. Install the Nix package manager, if you don't already have it. https://zero-to-nix.com/start/install is an excellent resource.

2. Enable `nix` command and [flakes](https://www.tweag.io/blog/2020-05-25-flakes/) features:

```bash
$ mkdir -p ~/.config/nix/ && printf 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf
```

3. Trust pre-built binaries (optional):

```bash
$ printf 'trusted-substituters = https://cache.garnix.io https://cache.nixos.org/\ntrusted-public-keys = cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g= cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=' | sudo tee -a /etc/nix/nix.conf && sudo systemctl restart nix-daemon
```

4. Alias `kneard-mgr` and use [`nix run`](https://determinate.systems/posts/nix-run) command:

```bash
$ printf 'alias kneard-mgr="nix run --refresh github:kuutamolabs/near-staking-knd --"' >> ~/.bashrc && source ~/.bashrc
```
5. Test the `kneard-mgr` command:

```bash
$ kneard-mgr --help
```

Answer ‘y’ to the four questions asked.
After some downloading you should see the help output.

```bash
Subcommand to run

Usage: kneard-mgr [OPTIONS] <COMMAND>

Commands:
  generate-config       Generate NixOS configuration
  install               Install Validator on a given machine. This will remove all data of the current system!
  dry-update            Upload update to host and show which actions would be performed on an update
  update                Update validator
  rollback              Rollback validator
  proxy                 Proxy remote rpc to local
  maintenance-restart   Ask Kuutamod to schedule a shutdown in maintenance windows, then it will be restart due to supervision by kneard
  maintenance-shutdown  Ask Kuutamod to schedule a shutdown in maintenance windows
  ssh                   SSH into a host
  help                  Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>  configuration file to load [env: KUUTAMO_CONFIG=] [default: kneard.toml]
      --yes              skip interactive dialogs by assuming the answer is yes
  -h, --help             Print help
  -V, --version          Print version
```

## New Solo node install 

1. New pool deployments can be done via the webapp UI `Get Started` flow at [near.kuutamo.app](https://near.kuutamo.app) - ([GitHub](https://github.com/kuutamolabs/near-staking-ui))

2. Download encrypted kuutamo app key file and config file (`kneard.toml`) via `Manage` button in UI:

3. Create a new directory and put the two files in it.

```
[you@workstation:~/my-near-validator-1/]$ ls
my-pool.pool.devnet.zip  kneard.toml
```

4. In this directory run:

```bash
$ kneard-mgr install
```

2. After this install finishes you can connect to the node.

```bash
$ kneard-mgr ssh
```

3. Follow the logs

```bash
[root@validator-00:~]$ journalctl -u kuutamod.service
```

## Node upgrades

In the folder:

```bash
$ kneard-mgr update
```


## Further Information

- [Install guide Google slides](https://docs.google.com/presentation/d/1SoXNkKUuYiH52rOb1lkEbmgKr2VEcJeYQAmpnLaOgtQ)
