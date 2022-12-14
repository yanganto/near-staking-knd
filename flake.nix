{
  description = "A supervisor for neard that implements failover for NEAR validators";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";

  inputs.srvos.url = "github:numtide/srvos";

  inputs.flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  inputs.core-contracts.url = "github:near/core-contracts";
  inputs.core-contracts.flake = false;
  inputs.fenix.url = "github:nix-community/fenix";
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";

  inputs.disko.url = "github:nix-community/disko";
  inputs.disko.inputs.nixpkgs.follows = "nixpkgs";

  inputs.nixos-remote.url = "github:numtide/nixos-remote/kuutamo";
  inputs.nixos-remote.inputs.nixpkgs.follows = "nixpkgs";
  inputs.nixos-remote.inputs.disko.follows = "disko";

  nixConfig.extra-substituters = [
    "https://cache.garnix.io"
  ];
  nixConfig.extra-trusted-public-keys = [
    "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
  ];

  outputs = { self, flake-parts, nixpkgs, fenix, ... }:
    flake-parts.lib.mkFlake { inherit self; } {
      imports = [
        ./nix/pkgs/flake-module.nix
        ./nix/modules/flake-module.nix
        ./nix/modules/tests/flake-module.nix
        ./nix/hosts/flake-module.nix
        ./nix/checks/flake-module.nix
        ./nix/shell.nix
      ];
      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
    };
}
