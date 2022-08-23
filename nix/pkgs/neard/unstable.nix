{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.29.0-rc.2";
  sha256 = "sha256-sqBJMHw6kKxZI9MWWW9cQL0Njn05lyyFDYQwt93veV8=";
  cargoSha256 = "sha256-gQUJudHlam7elVM87DdK0ou6NV8CROWU/OnLF9pvx6M=";
}
