{
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
}
