//! Supervises neard and participate in consul leader election.
//! The neard process of leader will get the validator key.

use crate::consul_client::{ConsulClient, ConsulError, ConsulSession};
use crate::exit_signal_handler::ExitSignalHandler;
use crate::leader_protocol::consul_leader_key;
use crate::near_client::NeardClient;
use crate::neard_process::{setup_validator, setup_voter, NeardProcess};
use crate::scoped_consul_session::ScopedConsulSession;
use crate::settings::Settings;
use anyhow::bail;
use anyhow::{Context, Result};
use futures_util::FutureExt;
use lazy_static::lazy_static;
use log::{info, warn};
use near_primitives::views::StatusResponse;
use nix::unistd;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::ops::Add;
use std::sync::Arc;
use tokio::time::{self, Duration, Instant};

lazy_static! {
    static ref STATE: IntGaugeVec = register_int_gauge_vec!(
        "kuutamod_state",
        "In what state our supervisor statemachine is",
        &["type"],
    )
    .unwrap();
}

/// How long a session is valid
const CONSUL_SESSION_TTL: Duration = Duration::from_secs(30);
/// How often to renew a consul session
const CONSUL_SESSION_RENEWAL: Duration = Duration::from_secs(10);
const CONSUL_SESSION_RENEWAL_ERROR: Duration = Duration::from_secs(10);
/// How much time we give neard to make it's `/status` endpoint available
const NEARD_STARTUP_TIMEOUT: Duration = Duration::from_secs(120);
/// How often we try to become consul leader (validator)
const CONSUL_ACQUIRE_LEADER_FREQUENCY: Duration = Duration::from_secs(1);
/// How long a leader will wait when it cannot update its consul session until steps down and stop doing validation
const CONSUL_LEADER_TIMEOUT: Duration = Duration::from_secs(25);
/// How often we query neard's `/status` endpoint
const NEARD_STATUS_FREQUENCY: Duration = Duration::from_secs(1);

// When adding states also update `initialize_state_gauge`
#[derive(PartialEq, Debug, Clone)]
enum StateType {
    Startup,
    Syncing,
    Registering,
    Voting,
    Validating,
    Shutdown,
}

fn initialize_state_gauge() {
    STATE
        .with_label_values(&[&StateType::Startup.to_string()])
        .set(1);
    STATE
        .with_label_values(&[&StateType::Syncing.to_string()])
        .set(0);
    STATE
        .with_label_values(&[&StateType::Registering.to_string()])
        .set(0);
    STATE
        .with_label_values(&[&StateType::Voting.to_string()])
        .set(0);
    STATE
        .with_label_values(&[&StateType::Validating.to_string()])
        .set(0);
    STATE
        .with_label_values(&[&StateType::Shutdown.to_string()])
        .set(0);
}

impl fmt::Display for StateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct StateMachine {
    inner: StateType,
    settings: Arc<Settings>,
    neard_process: Option<NeardProcess>,
    neard_client: NeardClient,
    consul_client: ConsulClient,
    consul_session: Option<ConsulSession>,
    exit_signal_handler: ExitSignalHandler,
    leader_metadata: HashMap<&'static str, String>,
    leader_key: String,
}

fn get_leader_metadata(node_id: &str) -> Result<HashMap<&'static str, String>> {
    let mut buf = [0u8; 64];
    let hostname_cstr = unistd::gethostname(&mut buf).context("Failed getting hostname")?;
    let hostname = hostname_cstr
        .to_str()
        .context("Hostname wasn't valid UTF-8")?;

    let mut metadata: HashMap<&str, String> = HashMap::new();
    metadata.insert("Hostname", hostname.into());
    metadata.insert("NodeId", node_id.into());
    Ok(metadata)
}

impl StateMachine {
    pub fn new(settings: &Arc<Settings>) -> Result<StateMachine> {
        Ok(StateMachine {
            inner: StateType::Startup,
            settings: Arc::clone(settings),
            neard_client: NeardClient::new(&format!(
                "http://localhost:{}",
                settings.near_rpc_addr.port()
            ))?,
            neard_process: None,
            consul_client: ConsulClient::new(
                &settings.consul_url,
                settings.consul_token.as_deref(),
            )
            .context("Failed to create consul client")?,
            consul_session: None,
            exit_signal_handler: ExitSignalHandler::new()
                .context("Failed to setup signal handler")?,
            leader_metadata: get_leader_metadata(&settings.node_id)
                .context("Failed to construct leader metadata")?,
            leader_key: consul_leader_key(&settings.account_id),
        })
    }
}

struct NeardStatus {
    next_try: Instant,
    continous_errors: u8,
}

impl NeardStatus {
    fn new() -> Self {
        Self {
            next_try: Instant::now(),
            continous_errors: 0,
        }
    }

    async fn query(&mut self, c: &NeardClient) -> Result<StatusResponse> {
        time::sleep_until(self.next_try).await;
        let res = c.status().await;
        self.next_try = Instant::now().add(NEARD_STATUS_FREQUENCY);
        res
    }

    async fn handle_neard_desyncs(&mut self, c: &NeardClient) -> StateType {
        while self.continous_errors < 3 {
            let status = self.query(c).await;
            match status {
                Ok(status) => {
                    self.continous_errors = 0;
                    if status.sync_info.syncing {
                        // node is synced fully with the network
                        return StateType::Syncing;
                    }
                }
                Err(err) => {
                    self.continous_errors += 1;
                    warn!("Cannot reach neard status api: {}", err);
                }
            }
        }
        StateType::Startup
    }
}

async fn wait_for_neard_exit(neard_process: Option<&mut NeardProcess>) {
    if let Some(p) = neard_process {
        match p.wait().await {
            Ok(res) => warn!("Neard finished unexpectly with {}. Check the logs above for potential error or panic messages from neard.", res),
            Err(err) => warn!("Cannot get status of neard process {}", err),
        }
    }
}

struct CreateSession {
    next_try: Instant,
    /// in seconds
    backoff: u64,
}

impl CreateSession {
    fn new() -> Self {
        Self {
            next_try: Instant::now(),
            backoff: 1,
        }
    }
    async fn run(&mut self, c: &ConsulClient, node_id: &str) -> Option<ConsulSession> {
        time::sleep_until(self.next_try).await;
        match c
            .create_session(node_id, CONSUL_SESSION_TTL.as_secs())
            .await
        {
            Ok(s) => Some(s),
            Err(e) => {
                warn!("Cannot reach consul: {}", e);
                self.backoff = std::cmp::min(self.backoff * 2, 5000);
                self.next_try = Instant::now().add(Duration::from_millis(self.backoff));
                None
            }
        }
    }
}

enum ChorumResult {
    IsFollower,
    IsMaster,
}

async fn acquire_key(
    c: &ConsulClient,
    leader_key: &str,
    metadata: &HashMap<&'static str, String>,
    session: &ConsulSession,
) -> ChorumResult {
    let res = c.acquire_key(leader_key, metadata, session);
    match res.await {
        // FIXME this could spam logs quite a bit (every second) -> add a rate limit for prints
        Err(e) => warn!("failed to contact consul: {}", e),
        Ok(is_master) => {
            if is_master {
                return ChorumResult::IsMaster;
            }
        }
    }
    ChorumResult::IsFollower
}

impl StateMachine {
    async fn handle_startup(&mut self) -> Result<StateType> {
        // give up after three times
        'restart: for _ in 0..3 {
            // stop old process if we still have one
            drop(self.neard_process.take());

            // if `execve` already fails, a retry likely won't solve the issue, so just error out in this case.
            self.neard_process = Some(setup_voter(&self.settings)?);
            let startup_timeout = time::Instant::now().add(NEARD_STARTUP_TIMEOUT);

            let mut neard_status = NeardStatus::new();

            loop {
                tokio::select! {
                    _ = wait_for_neard_exit(self.neard_process.as_mut()) => {
                        continue 'restart;
                    }
                    status = neard_status.query(&self.neard_client) => {
                        match status {
                            Ok(_) => {
                                return Ok(StateType::Syncing)
                            },
                            Err(e) => {
                                warn!("Failed to request neard status: {}", e)
                            },
                        }
                    }
                    // startup timeout
                    _ = time::sleep_until(startup_timeout) => {
                        warn!("Neard: Timeout on startup");
                        continue 'restart;
                    },
                    _ = self.exit_signal_handler.recv() => {
                        return Ok(StateType::Shutdown)
                    }
                }
            }
        }
        bail!("Could not start neard")
    }

    async fn handle_syncing(&mut self) -> Result<StateType> {
        let mut continous_errors = 0;
        let mut neard_status = NeardStatus::new();
        loop {
            tokio::select! {
                _ = wait_for_neard_exit(self.neard_process.as_mut()) => {
                    return Ok(StateType::Startup);
                }
                _ = self.exit_signal_handler.recv() => {
                    return Ok(StateType::Shutdown)
                }
                status = neard_status.query(&self.neard_client)=> {
                    match status {
                        Ok(status) => {
                            continous_errors = 0;
                            if !status.sync_info.syncing {
                                // node is synced fully with the network
                                return Ok(StateType::Registering);
                            }
                        }
                        Err(err) => {
                            warn!("Cannot reach neard status api: {}", err);
                            continous_errors += 1;
                            if continous_errors == 3 {
                                return Ok(StateType::Startup);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn handle_registering(&mut self) -> Result<StateType> {
        let mut create_session = CreateSession::new();
        let mut neard_status = NeardStatus::new();
        loop {
            tokio::select! {
                _ = wait_for_neard_exit(self.neard_process.as_mut()) => {
                    return Ok(StateType::Startup)
                },
                _ = self.exit_signal_handler.recv() => {
                    return Ok(StateType::Shutdown)
                }
                res = neard_status.handle_neard_desyncs(&self.neard_client) => {
                    return Ok(res)
                }
                // When we cancel this task, we might leak a consul session,
                // since it will however expire after 30s, this is fine.
                res = create_session.run(&self.consul_client, &self.settings.node_id) => {
                    self.consul_session = res;
                    if self.consul_session.is_some() {
                        return Ok(StateType::Voting)
                    }
                }
            }
        }
    }

    async fn handle_voting(&mut self) -> Result<StateType> {
        // this session needs to be manually moved or destroyed!
        let session = match self.consul_session.take() {
            Some(s) => ScopedConsulSession::new(&self.consul_client, s),
            None => {
                warn!("Got into validating state without consul session!");
                return Ok(StateType::Registering);
            }
        };

        let mut next_renewal = time::Instant::now().add(CONSUL_SESSION_RENEWAL);
        let mut next_acquire = time::Instant::now();
        let mut neard_status = NeardStatus::new();

        loop {
            tokio::select! {
                _ = wait_for_neard_exit(self.neard_process.as_mut()) => { return Ok(StateType::Startup) },
                // renew sessions every 10s
                _ = self.exit_signal_handler.recv() => {
                    session.destroy().await;
                    return Ok(StateType::Shutdown)
                }
                res = time::sleep_until(next_acquire).then(|()| acquire_key(&self.consul_client, &self.leader_key, &self.leader_metadata, session.borrow())) => {
                    if let ChorumResult::IsMaster = res {
                        // move back the session so that we can use in the validating state
                        self.consul_session = Some(session.into());
                        return Ok(StateType::Validating)
                    }
                    next_acquire = time::Instant::now().add(CONSUL_ACQUIRE_LEADER_FREQUENCY);
                }
                res = neard_status.handle_neard_desyncs(&self.neard_client) => {
                    session.destroy().await;
                    return Ok(res)
                }
                res = time::sleep_until(next_renewal).then(|()| self.consul_client.renew_session(session.borrow())) => {

                    if let Err(err) = res {
                        if let Some(&ConsulError::SessionNotFound) = err.downcast_ref::<ConsulError>() {
                            session.destroy().await;
                            return Ok(StateType::Registering)
                        }
                        warn!("failed to renew consul session: {}", err);
                        next_renewal = time::Instant::now().add(CONSUL_SESSION_RENEWAL_ERROR);
                    } else {
                        next_renewal = time::Instant::now().add(CONSUL_SESSION_RENEWAL);
                    };
                }
            }
        }
    }

    async fn handle_validating(&mut self) -> Result<StateType> {
        // this session needs to be manually moved or destroyed!
        let session = match self.consul_session.take() {
            Some(s) => ScopedConsulSession::new(&self.consul_client, s),
            None => {
                warn!("Got into validating state without consul session!");
                return Ok(StateType::Registering);
            }
        };

        // Stop neard that is not a validator.
        if let Some(p) = self.neard_process.take() {
            if let Err(e) = p.graceful_stop().context("Failed to stop validator") {
                session.destroy().await;
                return Err(e);
            }
        };
        let mut validator =
            match setup_validator(&self.settings).context("Failed to start validator") {
                Ok(v) => v,
                Err(e) => {
                    session.destroy().await;
                    return Err(e);
                }
            };

        let mut on_startup = true;
        let mut continous_errors = 0;
        let mut next_renewal = time::Instant::now().add(CONSUL_SESSION_RENEWAL);
        let mut session_expired = time::Instant::now().add(CONSUL_LEADER_TIMEOUT);
        let mut neard_status = NeardStatus::new();

        loop {
            tokio::select! {
                res = validator.process().wait() => {
                    match res {
                        Ok(res) => warn!("Neard finished unexpectly with {}", res),
                        Err(err) => warn!("Cannot get status of neard process {}", err),
                    }
                    drop(validator);
                    session.destroy().await;
                    return Ok(StateType::Startup)
                }
                _ = self.exit_signal_handler.recv() => {
                    drop(validator);
                    session.destroy().await;
                    return Ok(StateType::Shutdown)
                }
                res = neard_status.query(&self.neard_client) => {
                    match res {
                        Ok(status) => {
                            continous_errors = 0;
                            on_startup = false;
                            if status.sync_info.syncing {
                                // FIXME, we might want to add a threshold after which we step down here.
                                warn!("node is syncing!")
                            }
                        }
                        Err(err) => {
                            continous_errors += 1;
                            warn!("Cannot reach neard status api: {}", err);
                            if on_startup {
                                // On startup we give neard ~120s to make it's status api reachable.
                                // This is needed on testnet where the startup can take a long time.
                                if continous_errors == 120 {
                                    return Ok(StateType::Startup)
                                }
                            } else if continous_errors == 3 {
                                return Ok(StateType::Startup)
                            }
                        }
                    }
                }
                res = time::sleep_until(next_renewal).then(|()| self.consul_client.renew_session(session.borrow())) => {

                    if let Err(err) = res {
                        if let Some(&ConsulError::SessionNotFound) = err.downcast_ref::<ConsulError>() {
                            // no need to unregister an expired session
                            let _s : ConsulSession = session.into();
                            return Ok(StateType::Registering)
                        }
                        warn!("failed to renew consul session: {}", err);
                        next_renewal = time::Instant::now().add(CONSUL_SESSION_RENEWAL_ERROR);
                    } else {
                        next_renewal = time::Instant::now().add(CONSUL_SESSION_RENEWAL);
                        session_expired = time::Instant::now().add(CONSUL_LEADER_TIMEOUT);
                    };
                }
                _ = time::sleep_until(session_expired) => {
                    warn!("Lost connection to consul, step back");
                    // try to re-use our current session for voting
                    self.consul_session = Some(session.into());
                    return Ok(StateType::Voting)
                }
            }
        }
    }

    async fn next(&mut self) -> Result<StateType> {
        let new_state = match &self.inner {
            StateType::Startup => self
                .handle_startup()
                .await
                .context("Failed in startup state"),
            StateType::Syncing => self
                .handle_syncing()
                .await
                .context("Failed in syncing state"),
            StateType::Registering => self
                .handle_registering()
                .await
                .context("Failed in registering state"),
            StateType::Voting => self.handle_voting().await.context("Failed in voting state"),
            StateType::Validating => self
                .handle_validating()
                .await
                .context("Failed in validating state"),
            StateType::Shutdown => {
                bail!("Programming Error: next() should be not called if we are about to shutdown");
            }
        }?;
        if new_state != self.inner {
            // FIXME: This is not atomic!
            STATE.with_label_values(&[&self.inner.to_string()]).set(0);
            STATE.with_label_values(&[&new_state.to_string()]).set(1);
            info!("state changed: {:?} -> {:?}", self.inner, new_state)
        }
        self.inner = new_state;
        Ok(self.inner.clone())
    }
}

/// Runs neard and participate in consul leader election
pub async fn run_supervisor(settings: &Arc<Settings>) -> Result<()> {
    initialize_state_gauge();

    let mut state = StateMachine::new(settings).context("Failed to initialize state machine")?;

    while state.next().await? != StateType::Shutdown {}
    Ok(())
}
