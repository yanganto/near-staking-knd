{ stdenv, consul, kneard, neard, python3 }:
stdenv.mkDerivation {
  name = "kneard-unit-tests";
  src = ../../tests;
  nativeBuildInputs = [ consul neard kneard python3.pkgs.pytest ];
  doCheck = true;
  checkPhase = ''
    pytest -s .
  '';
  KUUTAMOD_BIN = "${kneard}/bin";
  NEARD_VERSION = "${neard.version}";
  installPhase = "touch $out";
}
