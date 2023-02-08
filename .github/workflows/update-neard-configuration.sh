#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR"

network=${1:-mainnet}

curl "https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/$network/config.json" | \
  jq 'setpath(["rpc", "enable_debug_rpc"]; true) | setpath(["telemetry", "endpoints"]; []) | setpath(["archive"]; false)' \
  > "$SCRIPT_DIR/../../nix/modules/neard/${network}/config.json"
