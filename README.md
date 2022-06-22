# kuutamod

kuutamod is a supervisor for neard that implements failover for [NEAR validators](https://near.org/validators/)

## Configuration:

kuutamod uses the following environment variables:

- `KUUTAMO_NODE_ID` (default: node), unique identifier for the current node (used in logging)
- `KUUTAMO_ACCOUNT_ID` (default: default), NEAR Account id of the validator.
   This ID will be used to acquire leadership in consul. It should be the same
   for all nodes that share the same validator key.
- `KUUTAMO_CONSUL_URL` (default: http://localhost:8500), url of the consul service that is used to reach consensus.
- `KUUTAMO_EXPORTER_ADDRESS` (default: 127.0.0.1:2233), address on which the local prometheus endpoint is exposed.
- `KUUTAMO_VALIDATOR_KEY`, (no default), path to near validator key, will
  fall back to `$CREDENTIALS_DIRECTORY/validator_key.json` if
  `KUUTAMO_VALIDATOR_KEY` is not set.
  
- `KUUTAMO_VALIDATOR_NODE_KEY`, (no default), path to near validator node key, will
  fall back to `$CREDENTIALS_DIRECTORY/validator_node_key.json` if
  `KUUTAMO_VALIDATOR_NODE_KEY` is not set.
  
- `KUUTAMO_VOTER_NODE_KEY`, (no default), path to near voter node key, will fall
  back to `$CREDENTIALS_DIRECTORY/voter_node_key.json` if `KUUTAMO_VOTER_NODE_KEY` is
  not set.  The voter node key should be unique, while the near validator node
  key should be the same on every host. The voter node key will be used by
  neard while the instance is not the validator.
  
- `KUUTAMO_NEARD_HOME` (default: `.`): where neard data is located, kuutamod expects neard configuration
  to be set up prior to start.
- `KUUTAMO_NEARD_BOOTNODES`: (default: None, optional) if provided, neard will
  use these nodes for bootstrapping connection to the network.

## Prometheus

kuutamod exports the following promethues metrics:

- `kuutamod_neard_restarts`: How often neard has been restarted
- `kuutamod_state`: In what state our supervisor statemachine is
- `kuutamod_uptime`: Time in milliseconds how long daemon is running
