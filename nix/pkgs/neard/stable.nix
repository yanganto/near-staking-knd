{ pkgs, buildPackages, rustToolchain_1_63 }:

let
  generic = pkgs.callPackage ./generic.nix { };
  neardRustPlatform = pkgs.callPackage buildPackages.makeRustPlatform {
    rustc = rustToolchain_1_63.rustc;
    cargo = rustToolchain_1_63.cargo;
  };
in
generic {
  version = "1.30.0-rc.4";
  sha256 = "sha256-0J6dJf/OJGL8avhIKIs1HlfsCi46B+Wv810qMQQtp3w=";
  cargoSha256 = "sha256-wlMzFs+1y3oDwa8wr94AEhG3dlVxYiE8SbirgKWlmxk=";
  inherit neardRustPlatform;
}
