{ lib, ... }: {
  systemd.network.enable = true;
  # don't take down the network for too long
  systemd.services.systemd-networkd.stopIfChanged = false;
  # Services that are only restarted might be not able to resolve when this is stopped before
  systemd.services.systemd-resolved.stopIfChanged = false;
  # often hangs
  systemd.services.systemd-networkd-wait-online.enable = lib.mkForce false;
  networking.dhcpcd.enable = false;
}
