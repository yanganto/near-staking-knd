{ rustPlatform
, lib
, runCommand
, nix
, openssh
, rsync
, cargoLock
, nixos-anywhere
, makeWrapper
, neard
, git
, nixos-rebuild
}:
# FIXME: refactor this repository to have multiple workspaces
rustPlatform.buildRustPackage {
  name = "kneard-mgr";
  # avoid trigger rebuilds if unrelated files are changed
  src = runCommand "src" { } ''
    install -D ${../../Cargo.toml} $out/Cargo.toml
    install -D ${../../Cargo.lock} $out/Cargo.lock
    install -D ${../../nix/modules/tests/validator_key.json} $out/nix/modules/tests/validator_key.json
    install -D ${../../nix/modules/tests/node_key.json} $out/nix/modules/tests/node_key.json
    cp -r ${../../src} $out/src
    cp -r ${../../build.rs} $out/build.rs
    pushd $out/src/deploy
    ls -la ../../nix/modules/tests/node_key.json
  '';
  inherit cargoLock;

  cargoBuildFlags = [ "--bin" "kneard-mgr" ];
  checkFlagsArray = [ "deploy::test_" ];

  nativeBuildInputs = [ makeWrapper ];

  # neard is for generating the key
  postInstall = ''
    wrapProgram $out/bin/kneard-mgr --prefix PATH : ${lib.makeBinPath [ nixos-anywhere nixos-rebuild nix git openssh rsync neard ]}
  '';

  checkInputs = [ nix ];

  doCheck = true;

  meta = with lib; {
    description = "A NEAR Staking node distribution by kuutamo";
    homepage = "https://github.com/kuutamolabs/near-staking-knd";
    license = licenses.asl20;
    platforms = platforms.unix;
  };
}
