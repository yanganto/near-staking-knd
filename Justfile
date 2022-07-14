# Create staking pool in localnet
# FIXME: this still throws some errors
create-staking-pool:
  #!/usr/bin/env bash
  public_key=$(jq -r .public_key .data/near/localnet/validator/validator_key.json)
  local-near call poolv1.owner create_staking_pool \
    '{"staking_pool_id": "validator","owner_id": "owner", "stake_public_key": "$public_key","reward_fee_fraction": {"numerator": 5, "denominator": 100}}' \
    --amount=30 \
    --accountId "owner" \
    --gas=300000000000000

# Upgrade neard package
upgrade-neard:
  nix-update --override-filename nix/pkgs/neard/stable.nix \
    --version-regex '^(\d+\.\d+\.\d+)$' \
    -f nix/pkgs/neard/nix-update.nix \
    --build --commit neard

# Upgrade neard-unstable package
upgrade-neard-unstable:
  nix-update --override-filename nix/pkgs/neard/unstable.nix \
    --version-regex '^(\d+\.\d+\.\d+(-rc.\d+)?)$' \
    -f nix/pkgs/neard/nix-update.nix \
    --build --commit neard-unstable --version=unstable

# Run kuutamo stack locally
run:
  hivemind

# Lint rust and python code
lint:
  cargo clippy --all-targets --all-features -- -D warnings
  mypy .

# Continously run cargo check as code changes
watch:
  cargo watch

# Run kuutamod integration tests
test:
  pytest -s tests

# To debug a single test using python's breakpoint
debug-test test:
  PYTHONBREAKPOINT=remote_pdb.set_trace REMOTE_PDB_HOST=127.0.0.1 REMOTE_PDB_PORT=4444 pytest -s {{test}}

# To attach to a breakpoint() call of debug-test
debug-repl:
   socat READLINE tcp:127.0.0.1:4444,forever,interval=2
