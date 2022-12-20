{ rustPlatform
, lib
, clippy
, openssl
, pkg-config
, mypy
, python3
, runCommand
, enableLint ? false
,
}:
rustPlatform.buildRustPackage ({
  name = "kuutamod" + lib.optionalString enableLint "-clippy";
  # avoid trigger rebuilds if unrelated files are changed
  src = runCommand "src" { } ''
    install -D ${../../Cargo.toml} $out/Cargo.toml
    install -D ${../../Cargo.lock} $out/Cargo.lock
    cp -r ${../../src} $out/src
  '';
  cargoLock.lockFile = ../../Cargo.lock;

  buildInputs = [ openssl ];
  nativeBuildInputs = [ pkg-config python3.pkgs.pytest ] ++ lib.optionals enableLint [ clippy mypy ];

  doCheck = false;

  meta = with lib; {
    description = "HA agent for neard";
    homepage = "https://github.com/kuutamoaps/kuutamocore";
    license = licenses.mit;
    maintainers = with maintainers; [ mic92 ];
    platforms = platforms.unix;
  };
}
  // lib.optionalAttrs enableLint {
  # we want python for this build
  src = runCommand "src" { } ''
    install -D ${../../Cargo.toml} $out/Cargo.toml
    install -D ${../../Cargo.lock} $out/Cargo.lock
    cp -r ${../../src} $out/src
    cp -r ${../../tests} $out/tests
    install -D ${../../pyproject.toml} $out/pyproject.toml
  '';
  buildPhase = ''
    mypy .
    cargo clippy --all-targets --all-features -- -D warnings
    if grep -R 'dbg!' ./src; then
      echo "use of dbg macro found in code!"
      false
    fi
  '';
  installPhase = ''
    touch $out
  '';
})
