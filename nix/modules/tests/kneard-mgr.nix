import ./lib.nix ({ self, pkgs, lib, ... }:
let
  inherit (self.packages.x86_64-linux) neard kneard-mgr;

  kexec-installer = self.inputs.nixos-images.packages.${pkgs.system}.kexec-installer-nixos-unstable;

  validator-system = self.nixosConfigurations.validator-00;

  dependencies = [
    validator-system.config.system.build.toplevel
    validator-system.config.system.build.diskoScript
    neard.rustChannelToml
  ] ++ builtins.map (i: i.outPath) (builtins.attrValues self.inputs);

  closureInfo = pkgs.closureInfo { rootPaths = dependencies; };

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

    # do not try to fetch stuff from the internet
    nix.settings = {
      experimental-features = [ "flakes" ];
      substituters = lib.mkForce [ ];
      hashed-mirrors = null;
      connect-timeout = 3;
      flake-registry = pkgs.writeText "flake-registry" ''{"flakes":[],"version":2}'';
    };

    environment.etc."install-closure".source = "${closureInfo}/store-paths";
    system.extraDependencies = dependencies;
  };
  qemu-common = import (pkgs.path + "/nixos/lib/qemu-common.nix") {
    inherit lib pkgs;
  };
  interfacesNumbered = config: lib.zipLists config.virtualisation.vlans (lib.range 1 255);
  getNicFlags = config: lib.flip lib.concatMap
    (interfacesNumbered config)
    ({ fst, snd }: qemu-common.qemuNICFlags snd fst config.virtualisation.test.nodeNumber);
in
{
  name = "install-nixos";
  nodes = {
    installer = { pkgs, ... }: {
      imports = [ shared ];
      systemd.network.networks."10-eth1".networkConfig.Address = "192.168.42.1/24";

      system.activationScripts.rsa-key = ''
        ${pkgs.coreutils}/bin/install -D -m600 ${./ssh-keys/ssh} /root/.ssh/id_rsa
      '';
    };
    installed = {
      imports = [ shared ];
      systemd.network.networks."10-eth1".networkConfig.Address = "192.168.42.2/24";

      virtualisation.emptyDiskImages = [ 4096 4096 ];
      virtualisation.memorySize = 4096;
      networking.nameservers = [ "127.0.0.1" ];
      services.openssh.enable = true;
      services.openssh.settings.UseDns = false;
      users.users.root.openssh.authorizedKeys.keyFiles = [ ./ssh-keys/ssh.pub ];
    };
  };
  testScript = { nodes, ... }:
    let
      tomlConfig = "${pkgs.runCommand "config" {} ''
        install -D ${./test-config.toml} $out/test-config.toml
        install -D ${./validator_key.json} $out/validator_key.json
        install -D ${./node_key.json} $out/node_key.json
      ''}/test-config.toml";
    in
    ''
      def create_test_machine(oldmachine=None, args={}): # taken from <nixpkgs/nixos/tests/installer.nix>
          machine = create_machine({
            "qemuFlags":
              '-cpu max -m 4024 -virtfs local,path=/nix/store,security_model=none,mount_tag=nix-store,'
              f' -drive file={oldmachine.state_dir}/empty0.qcow2,id=drive1,if=none,index=1,werror=report'
              ' -device virtio-blk-pci,drive=drive1'
              f' -drive file={oldmachine.state_dir}/empty1.qcow2,id=drive2,if=none,index=2,werror=report'
              ' -device virtio-blk-pci,drive=drive2'
              ' ${toString (getNicFlags nodes.installed)}'
          } | args)
          driver.machines.append(machine)
          return machine

      start_all()
      installed.wait_for_unit("sshd.service")
      installed.succeed("ip -c a >&2; ip -c r >&2")

      installer.wait_for_unit("network.target")
      installer.succeed("ping -c1 192.168.42.2")
      # our test config will read from here
      installer.succeed("cp -r ${self} /root/near-staking-knd")

      # closureInfo might return incorrect checksums, so we need to force-reregister our store paths
      installer.succeed("(echo ${neard}; echo; echo 0) | ${pkgs.nix}/bin/nix-store --register-validity --reregister")
      installer.succeed("${pkgs.nix}/bin/nix-store --verify-path ${neard}")

      installer.succeed("${lib.getExe kneard-mgr} --config ${tomlConfig} --yes install --debug --no-reboot --kexec-url ${kexec-installer}/nixos-kexec-installer-${pkgs.stdenv.hostPlatform.system}.tar.gz >&2")
      installer.succeed("ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no root@192.168.42.2 -- reboot >&2")
      installed.shutdown()

      new_machine = create_test_machine(oldmachine=installed, args={ "name": "after_install" })
      new_machine.start()
      hostname = new_machine.succeed("hostname").strip()
      assert "validator-00" == hostname, f"'validator-00' != '{hostname}'"

      installer.wait_until_succeeds("ssh -o StrictHostKeyChecking=no root@192.168.42.2 -- exit 0 >&2")

      new_machine.succeed("test -f /var/lib/secrets/node_key.json")
      new_machine.succeed("test -f /var/lib/secrets/validator_key.json")
      new_machine.succeed("rm /var/lib/secrets/validator_key.json")
      new_machine.wait_for_unit("consul.service")

      installer.succeed("${lib.getExe kneard-mgr} --config ${tomlConfig} --yes dry-update >&2")
      # redeploying uploads the key
      new_machine.succeed("test -f /var/lib/secrets/validator_key.json")

      installer.succeed("${lib.getExe kneard-mgr} --config ${tomlConfig} --yes update --immediately >&2")
      installer.succeed("${lib.getExe kneard-mgr} --config ${tomlConfig} --yes update --immediately >&2")
      # XXX find out how we can make persist more than one profile in our test
      #installer.succeed("${lib.getExe kneard-mgr} --config ${tomlConfig} --yes rollback --immediately >&2")

      hostname = installer.succeed("${lib.getExe kneard-mgr} --config ${tomlConfig} ssh hostname").strip()
      assert "validator-00" == hostname, f"'validator-00' != '{hostname}'"

      system_info = installer.succeed("${lib.getExe kneard-mgr} --config ${tomlConfig} system-info").strip()
      assert system_info.startswith("[validator-00]\nkneard-version: 0.3.0\ngit-sha:"), f"unexpected system info: {system_info}"
    '';
})
