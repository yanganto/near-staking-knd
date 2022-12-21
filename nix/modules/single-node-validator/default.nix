{ ... }: {
  imports = [
    ./network.nix
    ./hardware.nix
    ./consul.nix
    ./toml-mapping.nix
  ];

  system.stateVersion = "22.05";
}
