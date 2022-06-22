{ stdenv, consul, kuutamod, neard, python3 }:
stdenv.mkDerivation {
  name = "kuutamod-tests";
  src = ../../tests;
  nativeBuildInputs = [ consul neard kuutamod python3.pkgs.pytest ];
  doCheck = true;
  checkPhase = ''
    pytest -s .
  '';
  KUUTAMOD_BIN = "${kuutamod}/bin";
  installPhase = "touch $out";
}
