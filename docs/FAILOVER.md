# Primary-Secondary failover design

This page describes the internal kuutamod state machine used
to manage neard and failover to instances in case a failure happens.

## States

When starting up, kuutamod will go through the series of states before it's
promoted as a validator. Here is an overview of all states:

1. Startup:
  - Initial state
  - Start neard and wait for `/status` api to become available
2. Syncing:
  - Wait for client to catch up with the chain
3. Registering:
  - Monitor that neard is still in sync with the chain
  - Create a consul session
4. Voting:
  - Monitor that neard is in sync with the chain
  - Try to become leader of the validator key in consul,
    using the consul session from the previous state.
5. Validating:
  - In this state, neard will be restarted with the validator key
  - Only one validator instance should get into this state.

Kuutamod also exports the state it's currently in through its prometheus API:

```console
$ curl --silent http://localhost:2233/metrics | grep -E 'kuutamod_state'
# HELP kuutamod_state In what state our supervisor statemachine is
# TYPE kuutamod_state gauge
kuutamod_state{type="Registering"} 0
kuutamod_state{type="Shutdown"} 0
kuutamod_state{type="Startup"} 0
kuutamod_state{type="Syncing"} 0
kuutamod_state{type="Validating"} 1
kuutamod_state{type="Voting"} 0
```

In this case `kuutamod` is in `Validating` state.

## State transitions

In order to pass from one state to another, certain conditions must be
fulfilled. In the following we will go through all the possible transitions.

### 1. Startup -> 2. Syncing
 - If the http endpoint '/status' of neard is available.

### 2. Syncing -> 1. Startup
- When the neard process stops
- If the '/status' http endpoint is not available for 3 uninterrupted calls (one call per second).

### 2. Syncing -> 3. Registering
- When the '/status' endpoint indicates that synchronisation is complete

### 3. Registering -> 1. Startup
- When the Neard process is terminated
- When the '/status' endpoint is unreachable for 3 uninterrupted calls (one call per second)

### 3. Registering -> 2. Syncing
- When the '/status' endpoint indicates that neard is back in synchronisation status.

### 3. Registering -> 4. Voting
- When the consultation session has been successfully created

### 3. Voting -> 4. Validator
- If kuutamod can get a session lock for the key '/kuutamod-leader/<account_name>' in consul

### 4. Validator -> 3. Registering
- When consul reports that our session has expired

### 4. Validator -> 3. Startup
- When neard process stops

### 4. Validator -> 3. Voting
- If the session cannot be renewed for 20 seconds, restart neard without validator key
