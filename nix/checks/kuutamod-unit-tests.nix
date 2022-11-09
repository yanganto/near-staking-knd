{ stdenv, consul, kuutamod, neard, python3 }:
stdenv.mkDerivation {
  name = "kuutamod-unit-tests";
  src = ../../tests;
  nativeBuildInputs = [ consul neard kuutamod python3.pkgs.pytest ];
  doCheck = true;
  checkPhase = ''
    pytest -s .
  '';
  KUUTAMOD_BIN = "${kuutamod}/bin";
  NEARD_VERSION = "${neard.version}";
  installPhase = "touch $out";
}
