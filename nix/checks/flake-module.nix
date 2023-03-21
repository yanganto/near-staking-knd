_:
{
  perSystem = { self', pkgs, ... }:
    {
      checks = {
        kneard-unit-tests = pkgs.callPackage ./kneard-unit-tests.nix {
          inherit (self'.packages) neard;
          inherit (self'.packages) kneard;
        };
        kneard-unit-tests-unstable = self'.checks.kneard-unit-tests.override {
          neard = self'.packages.neard-unstable;
          kneard = self'.packages.kneard-unstable;
        };
        # for testing with binary releases
        #kneard-tests-bin = self'.checks.kneard-tests.override {
        #  neard = self'.packages.neard-bin;
        #};
        kneard-lint = self'.packages.kneard.override {
          enableLint = true;
        };
        kneard-unstable-lint = self'.packages.kneard.override {
          enableLint = true;
        };
      };
    };
}
