{ stdenv
, openssl
, autoPatchelfHook
,
}:
# to test incremental builds with our nixos tests
# use `cargo build -p neard && strip ./target/debug/neard` to produce smaller binary
stdenv.mkDerivation {
  name = "neard-bin";
  dontUnpack = true;
  buildInputs = [
    stdenv.cc.cc
    openssl
  ];
  nativeBuildInputs = [
    autoPatchelfHook
  ];
  installPhase = ''
    install -D -m755 ${./neard} $out/bin/neard
  '';
}
