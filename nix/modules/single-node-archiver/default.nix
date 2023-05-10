{ lib, ... }: {
  imports = [
    ../network.nix
    ../hardware.nix
    ../near-staking-analytics
    ../telegraf.nix
  ];

  system.stateVersion = "22.05";
  kuutamo.disko.disks = lib.mkDefault [ "/dev/nvme0n1" "/dev/nvme1n1" "/dev/nvme2n1" "/dev/nvme0n1" ];

  services.mongodb.enable = true;
  kuutamo.near-staking-analytics.enable = true;
  networking.firewall.allowedTCPPorts = [ 8080 ];
}
