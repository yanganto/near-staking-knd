{ pkgs, lib, ... }:
{
  imports = [
    ./default.nix
  ];
  # FIXME: This is hacky because it relies on shell...
  kuutamo.neard.s3.dataBackupDirectory = "s3://near-protocol-public/backups/mainnet/archive/$(${pkgs.awscli2}/bin/aws s3 --no-sign-request cp s3://near-protocol-public/backups/mainnet/archive/latest -)/";
  kuutamo.neard.configFile = pkgs.writeText "config.json" (
    builtins.toJSON (
      (lib.importJSON ../neard/mainnet/config.json) // { archiver = true; }
    )
  );
}
