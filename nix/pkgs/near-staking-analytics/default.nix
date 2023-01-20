{ fetchFromGitHub, stdenv, npmlock2nix, nodejs }:
stdenv.mkDerivation rec {
  name = "near-staking-analytics";
  src = fetchFromGitHub {
    owner = "kuutamolabs";
    repo = "near-staking-ui";
    rev = "ee405e928f98d3b356bd0a5b0af75af4d8b57bd9";
    hash = "sha256-F2nQhmqEF3xYsFEYWRxLxtzJKWEsA1Rp9k2UrRzuV7E=";
  };
  node_modules = npmlock2nix.v2.node_modules {
    src = src + "/backend";
  };
  installPhase = ''
    cd backend
    mkdir -p "$out/bin"
    mkdir -p "$out/share/near-staking-analytics"
    install -D package.json $out/share/near-staking-analytics/package.json
    cp -r src $out/share/near-staking-analytics/src
    ln -s ${node_modules}/node_modules "$out"/node_modules
    cat > "$out"/bin/near-staking-analytics <<EOF
    #!/bin/sh
    NODE_PATH=$out/node_modules ${nodejs}/bin/node $out/share/near-staking-analytics/src/index.js "\$@"
    EOF
    chmod +x "$out"/bin/near-staking-analytics
  '';
}
