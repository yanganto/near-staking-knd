{ rustPlatform
, lib
, runCommand
, nix
, openssh
, rsync
, cargoLock
, nixos-remote
, makeWrapper
, neard
}:
# FIXME: refactor this repository to have multiple workspaces
rustPlatform.buildRustPackage {
  name = "kuutamo";
  # avoid trigger rebuilds if unrelated files are changed
  src = runCommand "src" { } ''
    install -D ${../../Cargo.toml} $out/Cargo.toml
    install -D ${../../Cargo.lock} $out/Cargo.lock
    install -D ${../../nix/modules/tests/validator_key.json} $out/nix/modules/tests/validator_key.json
    install -D ${../../nix/modules/tests/node_key.json} $out/nix/modules/tests/node_key.json
    cp -r ${../../src} $out/src
    pushd $out/src/deploy
    ls -la ../../nix/modules/tests/node_key.json
  '';
  inherit cargoLock;

  cargoBuildFlags = [ "--bin" "kuutamo" ];
  checkFlagsArray = [ "deploy::test_" ];

  nativeBuildInputs = [ makeWrapper ];

  # neard is for generating the key
  postInstall = ''
    wrapProgram $out/bin/kuutamo --prefix PATH : ${lib.makeBinPath [ nixos-remote nix openssh rsync neard ]}
  '';

  checkInputs = [ nix ];

  doCheck = true;

  meta = with lib; {
    description = "A NEAR Staking node distribution by kuutamo";
    homepage = "https://github.com/kuutamolabs/near-staking-knd";
    license = licenses.asl20;
    maintainers = with maintainers; [ mic92 ];
    platforms = platforms.unix;
  };
}
