{ pkgs }:

let
  nodePackages = import ./default.nix {
    inherit pkgs;
    inherit (pkgs) system;
  };
in
nodePackages // {
  near-cli = nodePackages.near-cli.override {
    nativeBuildInputs = [
      pkgs.libusb1
      pkgs.nodePackages.prebuild-install
      pkgs.nodePackages.node-gyp-build
      pkgs.pkg-config
    ];
  };
}
