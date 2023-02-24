{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.31.0-rc.4";
  sha256 = "sha256-SZLMrUV0tSOsHlufM2Ycr5fswE3WJjjDmFcftfEH2nU=";
  cargoSha256 = "sha256-HRNsoHGqvArHBRIxGFlBZd362kDhNJt/X2Mr4r0jVQI=";
}
