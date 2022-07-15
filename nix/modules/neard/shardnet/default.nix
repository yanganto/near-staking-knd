{ config, lib, pkgs, ... }:

{
  imports = [
    ../.
  ];

  kuutamo.neard.configFile = lib.mkDefault ./config.json;
  kuutamo.neard.chainId = "shardnet";

  # If you set this to null, neard will download the Genesis file on first startup.
  kuutamo.neard.genesisFile = lib.mkDefault null;

  kuutamo.neard.s3.dataBackupTarball = lib.mkDefault "s3://build.openshards.io/stakewars/shardnet/data.tar.gz";
}
