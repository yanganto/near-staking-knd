# Primary-Secondary failover design

This is not the final design, but the plan for the first prototype.
Importantly the next iteration, should include also the neard rpc.

## States
1. Startup: 
  - Initial state
  - Start neard and wait for `/status` api to become available
2. Syncing: 
  - Wait for client to catch up with the chain
3. Client: 
  - Monitor that neard is in sync with the chain
  - Try to become leader of the validator key in consul
4. Validator: 
  - In this state, neard will be restarted with the validator key

## State transitions
TODO: TTLs are kind of arbitrary, find out what TTLs we need for NEAR.

### 1. Startup -> 2. Syncing
- Start neard and wait for `/status` api to become available

### 2. Syncing -> 1. Startup
- If neard process stops or it's 
- If `/status` api is done for 3 continous calls (one call per second)

### 2. Syncing -> 3. Client
- Monitor `/status` api and wait until neard indicates syncing is done

### 3. Client -> 1. Startup
- If neard process exits
- If `/status` api is done for 3 continous calls (one call per second)

### 3. Client -> 2. Syncing
- If `/status` api show that the neard is in syncing state again

### 3. Client -> 4. Validator
- Create a consul session with a 30s TTL 
- Renew consul session every 10
- Acquire session lock on `/kuutamod-leader` key in consul

### 4. Validator -> 3. Client
- Renew this session every 10s
- If cannot renewed for 20s seconds, restart neard without validator key
- If neard keeps crash-looping also step down.

# Some other random notes

- A graceful handover could be performed quicker than the consul ttl.
- If the current validator decides the step down and releases the lock, the next client would pick it up fairly quickly
- I would make the current master let decide to step down, if there are more healthy nodes in the cluster
  The information for that would be also stored in consul
- If the master sees that a different node has a signifantly better expected - produced diff, than it could step down.
