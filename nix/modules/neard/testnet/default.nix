{ lib, ... }:

{
  imports = [
    ../.
  ];

  kuutamo.neard.configFile = lib.mkDefault ./config.json;
  kuutamo.neard.chainId = "testnet";

  # If you set this to null, neard will download the Genesis file on first startup.
  kuutamo.neard.genesisFile = lib.mkDefault null;
}
