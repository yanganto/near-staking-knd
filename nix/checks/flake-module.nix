{ self, ... }:
{
  perSystem = { self', pkgs, ... }:
    {
      checks = {
        format = pkgs.callPackage ./check-format.nix {
          inherit self;
          inherit (self'.devShells.default) formatters;
        };
        kuutamod-unit-tests = pkgs.callPackage ./kuutamod-unit-tests.nix {
          neard = self'.packages.neard;
          kuutamod = self'.packages.kuutamod;
        };
        kuutamod-unit-tests-unstable = self'.checks.kuutamod-unit-tests.override {
          neard = self'.packages.neard-unstable;
        };
        # for testing with binary releases
        #kuutamod-tests-bin = self'.checks.kuutamod-tests.override {
        #  neard = self'.packages.neard-bin;
        #};
        kuutamod-lint = self'.packages.kuutamod.override {
          enableLint = true;
        };
      };
    };
}
