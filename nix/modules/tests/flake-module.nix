{ self, lib, ... }:

{
  perSystem = { pkgs, ... }:
    let
      makeTest = import (pkgs.path + "/nixos/tests/make-test-python.nix");

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
      };
    };
}
