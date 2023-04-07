{ near-staking-ui, stdenv, npmlock2nix, nodejs, python3 }:
stdenv.mkDerivation rec {
  name = "near-staking-analytics";
  src = near-staking-ui;
  node_modules = npmlock2nix.v2.node_modules {
    src = src + "/backend";
    buildInputs = [ python3 ];
    sourceOverrides = {
      buildRequirePatchShebangs = true;
      # bcrypt dependency
      "@mapbox/node-pre-gyp" = npmlock2nix.v2.packageRequirePatchShebangs;
    };
  };
  installPhase = ''
    cd backend
    mkdir -p "$out/bin"
    mkdir -p "$out/share/near-staking-analytics"
    install -D package.json $out/share/near-staking-analytics/package.json
    cp -r public $out/share/near-staking-analytics/public
    cp -r src $out/share/near-staking-analytics/src
    ln -s ${node_modules}/node_modules "$out"/node_modules
    cat > "$out"/bin/near-staking-analytics <<EOF
    #!/bin/sh
    NODE_PATH=$out/node_modules ${nodejs}/bin/node $out/share/near-staking-analytics/src/index.js "\$@"
    EOF
    chmod +x "$out"/bin/near-staking-analytics
  '';
}
