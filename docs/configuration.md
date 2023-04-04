# Configuration

If you are using our NixOS module, you can find all available options at the
time in the
[kneard](https://github.com/kuutamoaps/kneard/blob/main/nix/modules/kneard/default.nix)
module as well as the
[neard](https://github.com/kuutamoaps/kneard/blob/main/nix/modules/neard/default.nix)
module. If you plan to use kneard in other Linux distributions, we also list
here the underlying configuration options here.

kneard accepts all options to be either passed via commandline arguments or
via environment variables. Here is a list of all environment variables (you can
this information also by typing `kneard --help`):

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
  not set. The voter node key should be unique, while the near validator node
  key should be the same on every host. The voter node key will be used by
  neard while the instance is not the validator.

- `KUUTAMO_NEARD_HOME` (default: `.`): where neard data is located, kneard expects neard configuration
  to be set up prior to start.
- `KUUTAMO_NEARD_BOOTNODES`: (default: None, optional) if provided, neard will
  use these nodes for bootstrapping connection to the network.
  
- `KUUTAMO_CONSUL_URL` (default: http://localhost:8500, optional), the consul agent url 
- `KUUTAMO_CONSUL_TOKEN_FILE` (no default, optional), Consul token used for authentication, also see `https://www.consul.io/docs/security/acl/acl-tokens` 
- `KUUTAMO_PUBLIC_ADDRESS` Comma-separated list of ip addresses to be written
  to neard configuration on which the validator is *directly* reachable.
  Kuutamod will add the configured validator node key and port number of
  this node to these addresses.
