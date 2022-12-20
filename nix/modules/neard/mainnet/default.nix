{ lib, ... }:

{
  imports = [
    ../.
  ];
  kuutamo.neard.configFile = lib.mkDefault ./config.json;
  kuutamo.neard.genesisFile = lib.mkDefault ./genesis.json;
}
