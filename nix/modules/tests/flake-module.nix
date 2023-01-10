{ self, inputs, lib, ... }:

{
  perSystem = { pkgs, inputs', self', ... }:
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
      generated-flake = pkgs.runCommand "generated-flake" { } ''
        ${lib.getExe self'.packages.kuutamo} --config "${./test-config.toml}" generate-config "$out"
      '';
    in
    {
      checks = lib.optionalAttrs pkgs.stdenv.isLinux {
        kuutamod = import ./kuutamod.nix {
          inherit makeTest';
          inherit (self) nixosModules;
        };
        neard = import ./neard.nix {
          inherit makeTest';
          inherit (self) nixosModules;
        };
        generated-flake-is-same = pkgs.runCommand "generated-flake-is-same" { } ''
          if ! diff -Naur "${generated-flake}" "${./test-flake}"; then
            echo "Generated configuration in ./test-flake is no longer up-to-date!!" >&2
            echo "run the following command:" >&2
            echo "$ just ./nix/modules/tests/generate-test-flake" >&2
            exit 1
          fi
          touch $out
        '';
        install-nixos = pkgs.callPackage ./install-nixos.nix {
          inherit self makeTest' eval-config kexec-installer;
          diskoModule = inputs.disko.nixosModules.disko;
          validator-system = self.nixosConfigurations.validator-00;
          inherit (self'.packages) kuutamo;

          inherit (self) nixosModules;
        };
      };
    };
  flake = {
    nixosModules.qemu-test-profile = { modulesPath, ... }: {
      imports = [
        (modulesPath + "/testing/test-instrumentation.nix")
        (modulesPath + "/profiles/qemu-guest.nix")
      ];
    };
  } // import ./test-flake/configurations.nix {
    near-staking-knd = self;
  };
}
