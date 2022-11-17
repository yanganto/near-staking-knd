{ fetchFromGitHub
, fetchpatch
, zlib
, openssl
, pkg-config
, protobuf
, rustPlatform
, llvmPackages
, lib
, stdenv
, autoPatchelfHook
, darwin
}:
{ version, rev ? null, sha256, cargoSha256, cargoBuildFlags ? [ ], neardRustPlatform ? rustPlatform }:
# based on https://github.com/ZentriaMC/neard-nix/blob/master/neardtynix
neardRustPlatform.buildRustPackage rec {
  pname = "neard";
  inherit version;

  # https://github.com/near/nearcore/tags
  src = fetchFromGitHub {
    owner = "near";
    repo = "nearcore";
    # there is also a branch for this version number, so we need to be explicit
    rev = if rev == null then "refs/tags/${version}" else rev;
    inherit sha256;
  };

  inherit cargoSha256;

  patches = [ ];

  cargoPatches = [
    # Stateviewer has a test dependency on the wasm contracts.
    # Since we are not building tests, we can skip those.
    ./0001-make-near-test-contracts-optional.patch

    # - Expected shutdown
    #   - https://github.com/near/nearcore/pull/7872
    # - Maintenance RPC
    #   - https://github.com/near/nearcore/pull/7887
    (
      # This branch: https://github.com/kuutamolabs/nearcore/tree/shutdown-patch-1.29.1-patch
      lib.optional (lib.versionOlder version "1.30.0-rc.4") (
        fetchpatch {
          name = "shutdown-patch-1.29.0-patch";
          url = "https://github.com/kuutamolabs/nearcore/commit/6253b22eb1458e148c33652a93bdd39c3bc9167f.patch";
          sha256 = "sha256-mvnANYlhKrSlnNAWIF9WmgeZzvD1wMwzwith8TZkvlg=";
        }
      )
    )

    # - Expected shutdown
    #   - https://github.com/near/nearcore/pull/7872
    # - Maintenance RPC
    #   - https://github.com/near/nearcore/pull/7887
    (
      lib.optional (lib.versionAtLeast version "1.30.0-rc4") (
        fetchpatch {
          name = "shutdown-patch-1.30.0-rc.4-p2";
          url = "https://github.com/yanganto/nearcore/commit/500ec141a79b75911f4e0badb407082f7f4dc37f.patch";
          sha256 = "sha256-xFHGp9ELXrF2vuklNeioR/SpXrxEUZEJ97VGoozaGvY=";
        }
      )
    )
  ];

  postPatch = ''
    substituteInPlace neard/build.rs \
      --replace 'get_git_version()?' '"nix:${version}"'
  '';

  doInstallCheck = true;
  installCheckPhase = ''
    $out/bin/neard --version | grep -q "nix:${version}"
  '';

  CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1";
  CARGO_PROFILE_RELEASE_LTO = "fat";
  NEAR_RELEASE_BUILD = "release";
  inherit cargoBuildFlags;

  OPENSSL_NO_VENDOR = 1; # we want to link to OpenSSL provided by Nix

  buildAndTestSubdir = "neard";
  doCheck = false;

  buildInputs = [
    zlib
    openssl
  ] ++ lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.DiskArbitration;

  nativeBuildInputs = [
    pkg-config
    protobuf
  ];

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${llvmPackages.libclang.lib}/lib/clang/${lib.getVersion llvmPackages.clang}/include";

  meta = with lib; {
    description = "Reference client for NEAR Protocol";
    homepage = "https://github.com/near/nearcore";
    license = licenses.gpl3;
    maintainers = with maintainers; [ mic92 ];
    platforms = platforms.unix;
  };
}
