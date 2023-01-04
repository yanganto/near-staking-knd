{ self
, kuutamo
, openssh
, makeTest'
, validator-system
, kexec-installer
, stdenv
, lib
, ...
}:

let
  shared = {
    virtualisation.vlans = [ 1 ];
    systemd.network = {
      enable = true;

      networks."10-eth1" = {
        matchConfig.Name = "eth1";
        linkConfig.RequiredForOnline = "no";
      };
    };
    documentation.enable = false;
  };
in
makeTest' {
  name = "nixos-remote";
  nodes = {
    installer = { pkgs, ... }: {
      imports = [ shared ];
      systemd.network.networks."10-eth1".networkConfig.Address = "192.168.42.1/24";

      system.activationScripts.rsa-key = ''
        ${pkgs.coreutils}/bin/install -D -m600 ${./ssh-keys/ssh} /root/.ssh/id_rsa
      '';
      system.extraDependencies = [
        validator-system.config.system.build.toplevel
        validator-system.config.system.build.disko
        # make all flake inputs available
      ] ++ builtins.map (i: i.outPath) (builtins.attrValues self.inputs);
    };
    installed = {
      imports = [ shared ];
      systemd.network.networks."10-eth1".networkConfig.Address = "192.168.42.2/24";

      virtualisation.emptyDiskImages = [ 4096 4096 ];
      virtualisation.memorySize = 4096;
      services.openssh.enable = true;
      services.openssh.useDns = false;
      users.users.root.openssh.authorizedKeys.keyFiles = [ ./ssh-keys/ssh.pub ];
    };
  };
  testScript = ''
    def create_test_machine(oldmachine=None, args={}): # taken from <nixpkgs/nixos/tests/installer.nix>
        machine = create_machine({
          "qemuFlags":
            '-cpu max -m 4024 -virtfs local,path=/nix/store,security_model=none,mount_tag=nix-store,'
            f' -drive file={oldmachine.state_dir}/empty0.qcow2,id=drive1,if=none,index=1,werror=report'
            f' -device virtio-blk-pci,drive=drive1'
            f' -drive file={oldmachine.state_dir}/empty1.qcow2,id=drive2,if=none,index=2,werror=report'
            f' -device virtio-blk-pci,drive=drive2'
        } | args)
        driver.machines.append(machine)
        return machine

    start_all()
    installed.wait_for_unit("sshd.service")
    installed.succeed("ip a")

    installer.wait_for_unit("network.target")
    installer.succeed("ping -c1 192.168.42.2")
    # our test config will read from here
    installer.succeed("cp -r ${self} /root/near-staking-knd")

    installer.succeed("${lib.getExe kuutamo} --config ${./test-config.toml} --yes install --kexec-url ${kexec-installer}/nixos-kexec-installer-${stdenv.hostPlatform.system}.tar.gz >&2")
    installed.shutdown()

    new_machine = create_test_machine(oldmachine=installed, args={ "name": "after_install" })
    new_machine.start()
    hostname = new_machine.succeed("hostname").strip()
    assert "validator-00" == hostname, f"'validator-00' != '{hostname}'"
  '';
}
