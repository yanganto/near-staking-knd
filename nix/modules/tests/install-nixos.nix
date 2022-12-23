{ lib
, self
, kuutamo
, openssh
, makeTest'
, writeShellScriptBin
, validator-system
, kexec-installer
, stdenv
, ...
}:
makeTest' {
  name = "nixos-remote";
  nodes = {
    installer = {
      virtualisation.vlans = [ 1 ];
      systemd.network = {
        enable = true;

        networks."10-eth1" = {
          matchConfig.Name = "eth1";
          linkConfig.RequiredForOnline = "no";
          networkConfig.Address = "192.168.42.1/24";
        };
      };

      documentation.enable = false;
      environment.etc.sshKey = {
        source = ./ssh-keys/ssh;
        mode = "0600";
      };
      environment.systemPackages = [
        (writeShellScriptBin "run-kuutamo" ''
          set -x
          set -eu -o pipefail
          eval $(ssh-agent)
          ssh-add /etc/sshKey
          # our test config will read from here
          cp -r ${self} /root/near-staking-knd
          exec ${lib.getExe kuutamo} --config "${./test-config.toml}" --yes install --kexec-url ${kexec-installer}/nixos-kexec-installer-${stdenv.hostPlatform.system}.tar.gz
        '')
      ];
      programs.ssh.startAgent = true;
      system.extraDependencies = [
        validator-system.config.system.build.toplevel
        validator-system.config.system.build.disko
        # make all flake inputs available
      ] ++ builtins.map (i: i.outPath) (builtins.attrValues self.inputs);
    };
    installed = {
      virtualisation.vlans = [ 1 ];
      systemd.network = {
        enable = true;

        networks."10-eth1" = {
          matchConfig.Name = "eth1";
          linkConfig.RequiredForOnline = "no";
          networkConfig.Address = "192.168.42.2/24";
        };
      };

      virtualisation.emptyDiskImages = [ 4096 4096 ];
      virtualisation.memorySize = 4096;
      documentation.enable = false;
      services.openssh.enable = true;
      services.openssh.useDns = false;
      users.users.root.openssh.authorizedKeys.keyFiles = [ ./ssh-keys/ssh.pub ];
    };
  };
  testScript = ''
    def create_test_machine(oldmachine=None, args={}): # taken from <nixpkgs/nixos/tests/installer.nix>
        machine = create_machine({
          "qemuFlags":
            '-cpu max -m 1024 -virtfs local,path=/nix/store,security_model=none,mount_tag=nix-store,'
            f' -drive file={oldmachine.state_dir}/installed.qcow2,id=drive1,if=none,index=1,werror=report'
            f' -device virtio-blk-pci,drive=drive1',
        } | args)
        driver.machines.append(machine)
        return machine

    start_all()
    installed.wait_for_unit("sshd.service")
    installed.succeed("ip a")

    installer.wait_for_unit("network.target")
    installer.succeed("ping -c1 192.168.42.2")
    installer.succeed("run-kuutamo >&2")
    installed.shutdown()

    new_machine = create_test_machine(oldmachine=installed, args={ "name": "after_install" })
    new_machine.start()
    assert "nixos-remote" == new_machine.succeed("hostname").strip()
  '';
}
