{
  description = "A supervisor for neard that implements failover for NEAR validators";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";

  inputs.srvos.url = "github:numtide/srvos";
  inputs.srvos.inputs.nixpkgs.follows = "nixpkgs";

  inputs.treefmt-nix.url = "github:numtide/treefmt-nix";

  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  inputs.core-contracts.url = "github:near/core-contracts";
  inputs.core-contracts.flake = false;

  inputs.disko.url = "github:nix-community/disko";
  inputs.disko.inputs.nixpkgs.follows = "nixpkgs";

  inputs.nixos-remote.url = "github:numtide/nixos-remote/detect-nixos-installer";
  inputs.nixos-remote.inputs.nixpkgs.follows = "nixpkgs";
  inputs.nixos-remote.inputs.disko.follows = "disko";
  inputs.nixos-remote.inputs.nixos-images.follows = "nixos-images";
  inputs.nixos-remote.inputs.treefmt-nix.follows = "treefmt-nix";
  inputs.nixos-remote.inputs.flake-parts.follows = "flake-parts";

  inputs.nixos-images.url = "github:nix-community/nixos-images";

  nixConfig.extra-substituters = [
    "https://cache.garnix.io"
  ];
  nixConfig.extra-trusted-public-keys = [
    "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
  ];

  outputs = inputs @ { flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
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
