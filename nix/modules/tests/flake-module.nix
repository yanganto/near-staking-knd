{ self, config, lib, pkgs, ... }:

{
  perSystem = { config, self', inputs', pkgs, ... }:
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
      checks = {
        kuutamod = import ./kuutamod.nix { inherit makeTest'; };
        neard = import ./neard.nix { inherit makeTest'; };
      };
    };
}
