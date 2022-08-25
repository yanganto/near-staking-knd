consul: consul agent -server -bootstrap-expect 1 -bind 127.0.0.1 -data-dir .data/consul
# XXX contracts are currently not needed, but maybe later nice for testing
#setup-localnet: python3 ./scripts/setup-localnet.py && python3 ./scripts/deploy-contracts.py && sleep 99999999
setup-localnet: python3 ./scripts/setup-localnet.py && sleep 99999999
node0: ./scripts/wait-file .data/near/localnet/node0/config.json && neard --home .data/near/localnet/node0 run
node1: ./scripts/wait-file .data/near/localnet/node1/config.json && neard --home .data/near/localnet/node1 run --boot-nodes $(jq -r .public_key < .data/near/localnet/node0/node_key.json)@127.0.0.1:33301
node2: ./scripts/wait-file .data/near/localnet/node2/config.json && neard --home .data/near/localnet/node2 run --boot-nodes $(jq -r .public_key < .data/near/localnet/node0/node_key.json)@127.0.0.1:33301
# This node key is used for our validator node key
# node3: ./scripts/wait-file .data/near/localnet/node3/config.json && neard --home .data/near/localnet/node3 run --boot-nodes $(jq -r .public_key < .data/near/localnet/node0/node_key.json)@127.0.0.1:33301
