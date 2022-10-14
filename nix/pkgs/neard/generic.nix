{ fetchFromGitHub
, zlib
, openssl
, pkg-config
, protobuf
, llvmPackages
, rustPlatform
, lib
, stdenv
, autoPatchelfHook
, darwin
}:

{ version, rev ? null, sha256, cargoSha256, cargoBuildFlags ? [ ] }:
# based on https://github.com/ZentriaMC/neard-nix/blob/master/neard.nix
rustPlatform.buildRustPackage rec {
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

  # On nixos the nix-daemon limits files to 4096 by default...
  # In our tests we probably don't need more than that...
  # However neard does not respect our store configuration in all cases.
  # Also see https://github.com/near/nearcore/issues/6857
  #
  # This should be fixed in https://github.com/near/nearcore/pull/6858
  patches = lib.optional (version == "1.28.1") ./0001-reduce-max_open_files-when-checking-version-v1.28.0.patch;

  # Stateviewer has a test dependency on the wasm contracts.
  # Since we are not building tests, we can skip those.
  cargoPatches = [ ./0001-make-near-test-contracts-optional.patch ];

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
