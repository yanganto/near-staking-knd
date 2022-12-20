{ self, inputs, lib, ... }:

{
  perSystem = { pkgs, inputs', ... }:
    let
      makeTest = import (pkgs.path + "/nixos/tests/make-test-python.nix");
      eval-config = import (pkgs.path + "/nixos/lib/eval-config.nix");
      kexec-installer = inputs'.nixos-images.packages.kexec-installer-nixos-unstable;

      makeTest' = test: (makeTest test {
        inherit pkgs;
        inherit (pkgs) system;
        specialArgs = self.lib.flakeSpecialArgs {
          inherit (pkgs) system;
        };
      }).test;
    in
    {
      checks = lib.optionalAttrs (pkgs.stdenv.isLinux) {
        kuutamod = import ./kuutamod.nix {
          inherit makeTest';
          inherit (self) nixosModules;
        };
        neard = import ./neard.nix {
          inherit makeTest';
          inherit (self) nixosModules;
        };
        install-nixos = pkgs.callPackage ./install-nixos.nix {
          inherit makeTest' eval-config kexec-installer;
          diskoModule = inputs.disko.nixosModules.disko;
          inherit (inputs'.nixos-remote.packages) nixos-remote;

          inherit (self) nixosModules;
        };
      };
    };
}
