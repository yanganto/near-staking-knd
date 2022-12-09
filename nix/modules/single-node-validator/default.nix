{ lib, pkgs, config, ... }: {
  networking.hostName = "nixos";

  imports = [
    # FIXME: this should be provided by kuutamoctl
    ./users.nix
    ./network.nix
  ];

  # FIXME: this should be provided by kuutamoctl
  kuutamo.network.ipv4.address = "199.127.63.197";
  kuutamo.network.ipv4.gateway = "199.127.63.1";
  kuutamo.network.ipv6.address = "2605:9880:400:700:8:b10c:1932:3224";
  kuutamo.network.ipv6.gateway = "2605:9880:400::1";

  # Single node consul server. Just needed for kuutamo here
  services.consul = {
    interface.bind = "lo";
    extraConfig = {
      server = true;
      bootstrap_expect = 1;
    };
  };

  boot.initrd.availableKernelModules = [
    "xhci_pci"
    "ahci"
    "nvme"
  ];

  disko.devices = import ./raid-config.nix {
    raidLevel = 0;
  };

  # / is a mirror raid
  boot.loader.grub.devices = [ "/dev/nvme0n1" "/dev/nvme1n1" ];
  # for mdraid 1.1
  boot.loader.grub.extraConfig = "insmod mdraid1x";
  boot.loader.grub.enable = true;
  boot.loader.grub.version = 2;

  # srvos limits ssh keys usage.
  services.openssh.authorizedKeysFiles = lib.mkForce [
    "/etc/ssh/authorized_keys.d/%u"
    "%h/.ssh/authorized_keys"
  ];

  # FIXME: how to upload these?
  kuutamo.kuutamod.validatorKeyFile = "/var/lib/secrets/validator_key.json";
  kuutamo.kuutamod.validatorNodeKeyFile = "/var/lib/secrets/node_key.json";


  system.stateVersion = "22.05";
}
