{ stdenv, rocksdb, gflags, jemalloc, lib, removeReferencesTo }:

stdenv.mkDerivation rec {
  pname = "rocksdb-tools";
  inherit (rocksdb) version src propagatedBuildInputs;

  nativeBuildInputs = rocksdb.nativeBuildInputs ++ [ removeReferencesTo ];

  cmakeFlags = [
    "-DPORTABLE=1"
    "-DWITH_JEMALLOC=${if stdenv.hostPlatform.isLinux then "1" else "0"}"
    "-DWITH_JNI=0"
    "-DWITH_BENCHMARK_TOOLS=1"
    "-DWITH_TESTS=0"
    "-DWITH_TOOLS=0"
    "-DWITH_BZ2=1"
    "-DWITH_LZ4=1"
    "-DWITH_SNAPPY=1"
    "-DWITH_ZLIB=1"
    "-DWITH_ZSTD=1"
    "-DWITH_GFLAGS=1"
    "-DUSE_RTTI=1"
    "-DROCKSDB_INSTALL_ON_WINDOWS=YES" # harmless elsewhere
    "-DROCKSDB_BUILD_SHARED=1"
    (lib.optional (stdenv.hostPlatform.isx86 && stdenv.hostPlatform.isLinux) "-DFORCE_SSE42=1")
  ];

  buildInputs = [ gflags ] ++ lib.optional (stdenv.hostPlatform.isLinux) jemalloc;

  makeFlags = [ "db_bench" ];

  installPhase = ''
    runHook preInstall
    mkdir -p $out/lib
    for f in librocksdb.so*; do
      cp -a $f $out/lib
    done
    install -D ./tools/sst_dump -m755 $out/bin/sst_dump
    install -D ./db_bench -m755 $out/bin/db_bench

    # remove rpath references to /build
    sed -i -e "s!/build/source!/XXXXXXXXXXXX!" $out/bin/*

    runHook postInstall
  '';
  NIX_LDFLAGS = "-rpath ${placeholder "out"}/lib";

  meta = with stdenv.lib; {
    description = "RocksDB benchmark";
    inherit (rocksdb.meta) homepage license platforms;
  };
}
