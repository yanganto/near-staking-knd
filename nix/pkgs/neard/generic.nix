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
}:

{ version, sha256, cargoSha256 }:
# based on https://github.com/ZentriaMC/neard-nix/blob/master/neard.nix
rustPlatform.buildRustPackage rec {
  pname = "neard";
  inherit version;

  # https://github.com/near/nearcore/tags
  src = fetchFromGitHub {
    owner = "near";
    repo = "nearcore";
    # there is also a branch for this version number, so we need to be explicit
    rev = "refs/tags/${version}";
    inherit sha256;
  };

  inherit cargoSha256;

  # On nixos the nix-daemon limits files to 4096 by default...
  # In our tests we probably don't need more than that...
  # However neard does not respect our store configuration in all cases.
  # Also see https://github.com/near/nearcore/issues/6857
  patches =
    if (version == "1.26.1") then [
      ./0001-relax-ulimit-check-for-nix-sandbox-build.patch
    ] else [
      ./0001-reduce-max_open_files-when-checking-version.patch
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

  OPENSSL_NO_VENDOR = 1; # we want to link to OpenSSL provided by Nix

  buildAndTestSubdir = "neard";
  doCheck = false;

  buildInputs = [
    zlib
    openssl
  ];

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
