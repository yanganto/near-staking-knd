{ pkgs, ... }:
{
  imports = [
    ./default.nix
  ];
  # FIXME: This is hacky because it relies on shell...
  kuutamo.neard.s3.dataBackupDirectory = "s3://near-protocol-public/backups/testnet/rpc/$(${pkgs.awscli2}/bin/aws s3 --no-sign-request cp s3://near-protocol-public/backups/testnet/rpc/latest -)/";
  kuutamo.exporter.externalRpc = "https://rpc.testnet.near.org";
}
