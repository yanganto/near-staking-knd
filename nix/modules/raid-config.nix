{ config
, lib
, ...
}:
let
  biosBoot = {
    part-type = "primary";
    start = "0MB";
    end = "1MB";
    name = "boot";
    flags = [ "bios_grub" ];
  };

  efiBoot = {
    name = "ESP";
    start = "1MB";
    end = "500MB";
    bootable = true;
    content = {
      type = "mdraid";
      name = "boot";
    };
  };

  raidPart = {
    part-type = "primary";
    name = "raid-root";
    start = "500MB";
    end = "100%";
    bootable = true;
    content = {
      type = "mdraid";
      name = "root";
    };
  };
in
{
  disko.devices = {
    disk = lib.genAttrs config.kuutamo.disko.disks (disk: {
      type = "disk";
      device = disk;
      content = {
        type = "table";
        format = "gpt";
        partitions = [
          biosBoot
          efiBoot
          raidPart
        ];
      };
    });

    mdadm = {
      boot = {
        type = "mdadm";
        # if one disk fails we can boot at least a kernel and show what is going on.
        level = 1;
        # metadata 1.0 so we can use it as an esp partition
        metadata = "1.0";
        content = {
          type = "filesystem";
          format = "vfat";
          mountpoint = "/boot";
        };
      };
      root = {
        type = "mdadm";
        level = config.kuutamo.disko.raidLevel;
        content = {
          type = "filesystem";
          format = "ext4";
          mountpoint = "/";
        };
      };
    };
  };
}
