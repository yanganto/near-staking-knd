{ fetchpatch, nix-update }:

nix-update.overrideAttrs (old: {
  patches = [
    (fetchpatch {
      url = "https://github.com/Mic92/nix-update/commit/438badbb012d3ff295ae396d952f81a9e5f290cb.patch";
      sha256 = "sha256-6YK16vOjFoS7nsLPFZdvbG6zXXibneyL/I0ATykjSOY=";
    })
    # https://github.com/Mic92/nix-update/pull/93
    (fetchpatch {
      url = "https://github.com/Mic92/nix-update/commit/7afa43c32efeaaccf4adb146c27819ce52a034bf.patch";
      sha256 = "sha256-7b5HHuD/PhlRNV/6j+3LvKay39bq0XGpMdKdtAWCqL4=";
    })
  ];
})
