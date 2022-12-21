{ rustPlatform
, lib
, runCommand
, nix
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
  cargoLock.lockFile = ../../Cargo.lock;

  cargoBuildFlags = [ "--bin" "kuutamo" ];
  checkFlagsArray = [ "deploy::test_" ];

  checkInputs = [ nix ];

  doCheck = true;

  meta = with lib; {
    description = "Command-line for setting up validators";
    homepage = "https://github.com/kuutamoaps/kuutamocore";
    license = licenses.mit;
    maintainers = with maintainers; [ mic92 ];
    platforms = platforms.unix;
  };
}
