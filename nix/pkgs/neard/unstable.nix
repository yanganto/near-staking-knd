{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.29.0-rc.1";
  sha256 = "sha256-+NTmTkgn1oCbLfIULkaixItS8GP2xTd+Xxt/D+avkz8=";
  cargoSha256 = "sha256-97Y6550qJFIN4yDN/yvA5FmwyBohXoykbLZyBKUQ1q4=";
}
