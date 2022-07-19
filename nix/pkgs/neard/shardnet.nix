{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "2022-07-18";
  rev = "8448ad1ebf27731a43397686103aa5277e7f2fcf";
  sha256 = "sha256-cbaFu6FehIB3MeERfsgy9f+afRTYSbwGpKigk410Kxc=";
  cargoSha256 = "sha256-Hn1Ws/+Jn20Z//5gTm8DTfKF2a4uuZ2q/C8zzo5TFPg=";
  cargoBuildFlags = [
    "--features=shardnet"
  ];
}
