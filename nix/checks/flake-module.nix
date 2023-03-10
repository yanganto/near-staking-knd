_:
{
  perSystem = { self', pkgs, ... }:
    {
      checks = {
        kuutamod-unit-tests = pkgs.callPackage ./kuutamod-unit-tests.nix {
          inherit (self'.packages) neard;
          inherit (self'.packages) kuutamod;
        };
        kuutamod-unit-tests-unstable = self'.checks.kuutamod-unit-tests.override {
          neard = self'.packages.neard-unstable;
          kuutamod = self'.packages.kuutamod-unstable;
        };
        # for testing with binary releases
        #kuutamod-tests-bin = self'.checks.kuutamod-tests.override {
        #  neard = self'.packages.neard-bin;
        #};
        kuutamod-lint = self'.packages.kuutamod.override {
          enableLint = true;
        };
        kuutamod-unstable-lint = self'.packages.kuutamod.override {
          enableLint = true;
        };
      };
    };
}
