{ pkgs }:

let
  generic = pkgs.callPackage ./generic.nix { };
in
generic {
  version = "1.30.0-rc.4";
  sha256 = "sha256-0J6dJf/OJGL8avhIKIs1HlfsCi46B+Wv810qMQQtp3w=";
  cargoSha256 = "sha256-X4hU9rqHQ9gcT2rQaKhY9o3flVZDO2w7gM8s6GQBHec=";
}
