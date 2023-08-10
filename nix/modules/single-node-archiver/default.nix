{ lib, pkgs, ... }: {
  imports = [
    ../network.nix
    ../hardware.nix
    ../near-staking-analytics
    ../near-prometheus-exporter.nix
  ];

  system.stateVersion = "22.05";
  kuutamo.disko.disks = lib.mkDefault [ "/dev/nvme0n1" "/dev/nvme1n1" "/dev/nvme2n1" "/dev/nvme3n1" ];

  services.mongodb = {
    enable = true;
    package = pkgs.mongodb.overrideAttrs (_: {
      meta = { };
      hardeningDisable = [ "fortify3" ];
    });
  };
  kuutamo.near-staking-analytics = {
    enable = true;
    backupLocation = "/root/backup";
  };
  networking.firewall.allowedTCPPorts = [ 8080 ];
}
