{ fetchFromGitHub
, fetchurl
, fetchpatch
, zlib
, openssl
, pkg-config
, protobuf
, llvmPackages
, lib
, stdenv
, darwin
, makeRustPlatform
}:
{ ver, rev ? null, sha256, cargoSha256, cargoBuildFlags ? [ ], toolchain, toolchainFile, toolchainChecksum, neardPatches ? [ ], revisionNumber ? null }:
let
  rustPlatform = makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
  toolchainToml = builtins.fromTOML (builtins.readFile toolchainFile);
  rustChannelToml = fetchurl {
    url = "https://static.rust-lang.org/dist/channel-rust-${toolchainToml.toolchain.channel}.toml";
    sha256 = toolchainChecksum;
  };
in
# based on https://github.com/ZentriaMC/neard-nix/blob/master/neardtynix
rustPlatform.buildRustPackage rec {
  pname = "neard";
  version = "${ver}${lib.optionalString (revisionNumber != null) "-rev${revisionNumber}"}";

  # https://github.com/near/nearcore/tags
  src = fetchFromGitHub {
    owner = "near";
    repo = "nearcore";
    # there is also a branch for this version number, so we need to be explicit
    rev = "refs/tags/${version}";
    inherit sha256;
  };

  inherit cargoSha256;

  patches = neardPatches;

  cargoPatches = [
    # Remove test dependency on contract
    # Since we are not building tests, we can skip those.
    (
      lib.optional (lib.versionOlder version "1.32.0") (
        ./0001-rm-near-test-contracts.patch
      )
    )
    (
      lib.optional (lib.versionAtLeast version "1.32.0") (
        ./0001-rm-near-test-contracts-1.32.patch
      )
    )

    ./0002-rocksdb-max-open.patch

    (
      lib.optional (lib.versionOlder version "1.32.0") (
        fetchpatch
          {
            name = "maintenance_patch-1.31.0-rc.5";
            url = "https://github.com/kuutamolabs/nearcore/commit/e045bee17716140e53dcb53bcae43bc4f3c4d8d7.patch";
            sha256 = "sha256-fKX+LvASNa0FR97qquUxIDXv91KKYKMaKRTNPuQF41w=";
          }
      )
    )
    (
      lib.optional (lib.versionAtLeast version "1.32.0") (
        ./0003-expected-shutdown-metrix.patch
      )
    )
  ];

  passthru = {
    # used in tests for offline evaluation
    inherit rustChannelToml;
  };

  postPatch = ''
    substituteInPlace neard/build.rs \
      --replace 'get_git_version()?' '"nix:${version}"'
  '';

  doInstallCheck = true;
  installCheckPhase = ''
    $out/bin/neard --version | grep -q "nix:${version}"
  '';
  preBuild = ''
    diff -q ${toolchainFile} ./rust-toolchain.toml > /dev/null || {
      echo -e "\033[0;1;31mERROR: ${toolchainFile} differs with ./rust-toolchain.toml. \033[0m" >&2
      echo -e "\033[0;1;31mPlease update nix/pkgs/neard/unstable-rust-toolchain.toml or nix/pkgs/neard/unstable-rust-toolchain.toml\033[0m" >&2
      exit 1
    }
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
    toolchain
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
