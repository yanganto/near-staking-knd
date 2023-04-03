#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR"

network=${1:-mainnet}

if [[ $network == "mainnet" ]]
then
    curl "https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/$network/config.json" | \
      jq 'setpath(["rpc", "enable_debug_rpc"]; true) | setpath(["telemetry", "endpoints"]; []) | setpath(["archive"]; false) | delpaths([["network", "reconnect_delay"], ["network", "external_address"] ])' \
      > "$SCRIPT_DIR/../../nix/modules/neard/${network}/config.json"
else
    curl "https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/$network/config.json" | \
      jq 'setpath(["rpc", "enable_debug_rpc"]; true) | setpath(["telemetry", "endpoints"]; []) | setpath(["archive"]; false) | delpaths([["network", "reconnect_delay"], ["network", "external_address"] ]) | setpath(["network", "tier1_enable_outbound"]; true) ' \
      > "$SCRIPT_DIR/../../nix/modules/neard/${network}/config.json"
fi
