{ rustPlatform
, lib
, runCommand
, nix
, cargoLock
, nixos-remote
, makeWrapper
}:
# FIXME: refactor this repository to have multiple workspaces
rustPlatform.buildRustPackage {
  name = "kuutamo";
  # avoid trigger rebuilds if unrelated files are changed
  src = runCommand "src" { } ''
    install -D ${../../Cargo.toml} $out/Cargo.toml
    install -D ${../../Cargo.lock} $out/Cargo.lock
    cp -r ${../../src} $out/src
  '';
  inherit cargoLock;

  cargoBuildFlags = [ "--bin" "kuutamo" ];
  checkFlagsArray = [ "deploy::test_" ];

  nativeBuildInputs = [ makeWrapper ];

  postInstall = ''
    wrapProgram $out/bin/kuutamo --prefix PATH : ${lib.makeBinPath [ nixos-remote ]}
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
