{ pkgs, buildPackages, rustToolchain_1_63 }:

let
  generic = pkgs.callPackage ./generic.nix { };
  neardRustPlatform = pkgs.callPackage buildPackages.makeRustPlatform {
    rustc = rustToolchain_1_63.rustc;
    cargo = rustToolchain_1_63.cargo;
  };
in
generic {
  version = "1.29.2";
  sha256 = "sha256-dVju9emwTqNQCYST4HuwSWdafM0yxVS3JXXJqCdFEpc=";
  cargoSha256 = "sha256-HV+TAXVYWW7mjNpYCT7G6kh/xNxHq+saGbym5LiKn2Y=";
  inherit neardRustPlatform;
}
