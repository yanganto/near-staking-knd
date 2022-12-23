{ lib, config, ... }: {
  options = {
    kuutamo.disko.raidLevel = lib.mkOption {
      type = lib.types.int;
      default = 0;
      description = "Raid level used for the system disks";
    };
    # Upstream this?
    kuutamo.disko.disks = lib.mkOption {
      type = lib.types.listOf lib.types.path;
      default = [ "/dev/nvme1n1" "/dev/nvme0n1" ];
      description = lib.mdDoc "Disks formatted by disko";
    };
  };
  imports = [ ./raid-config.nix ];

  config = {
    boot.initrd.availableKernelModules = [
      "xhci_pci"
      "ahci"
      "nvme"
    ];
    # / is a mirror raid
    boot.loader.grub.devices = config.kuutamo.disko.disks;
    # for mdraid 1.1
    boot.loader.grub.extraConfig = "insmod mdraid1x";
    boot.loader.grub.enable = true;
    boot.loader.grub.version = 2;
  };
}
