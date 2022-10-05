{ stdenv, rocksdb, gflags, jemalloc, lib }:

stdenv.mkDerivation rec {
  pname = "db_bench";
  inherit (rocksdb) version src nativeBuildInputs propagatedBuildInputs;

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
    "-DROCKSDB_BUILD_SHARED=0"
    (lib.optional (stdenv.hostPlatform.isx86 && stdenv.hostPlatform.isLinux) "-DFORCE_SSE42=1")
  ];

  buildInputs = [ gflags ] ++ lib.optional (stdenv.hostPlatform.isLinux) jemalloc;

  makeFlags = [ "db_bench" ];

  installPhase = ''
    runHook preInstall
    install -D ./db_bench -m755 $out/bin/db_bench
    runHook postInstall
  '';

  meta = with stdenv.lib; {
    description = "RocksDB benchmark";
    inherit (rocksdb.meta) homepage license platforms;
  };
}
