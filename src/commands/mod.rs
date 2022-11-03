//! Kuutamo command utils
//! Command handler helps to listen command and excute command and return new state if needed
//! Kuutamo client helps to send command
mod maintenance_shutdown;

use crate::settings::Settings;
use crate::supervisor::StateType;
use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use log::{info, warn};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use std::fs::{metadata, remove_file};
use std::path::PathBuf;
use tokio::io::Interest;
use tokio::net::UnixListener;
use tokio::net::UnixStream;
use tokio_stream::wrappers::UnixListenerStream;

#[allow(clippy::derive_partial_eq_without_eq)]
/// The command for kuutamod to control the behavior of neard
#[derive(clap::Subcommand, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    /// Setup a maintenance window and restart, will not change the state
    MaintenanceShutdown,

    /// Show the current voted validator
    ActiveValidator,
}

/// Command handler help to listen command and excute command and return new state if needed
pub(crate) struct CommandHander<'a> {
    current_state: &'a StateType,
    dynamic_conf_path: PathBuf,
    socket_path: PathBuf,
    ctl_socket: UnixListenerStream,
    near_rpc_port: u16,
    neard_process_id: Option<Pid>,
}

impl<'a> CommandHander<'a> {
    /// New command hander to handle commands
    pub fn new(
        current_state: &'a StateType,
        settings: &Settings,
        neard_process_id: Option<Pid>,
    ) -> Result<Self> {
        let socket_path = settings.neard_home.join("kuutamod.ctl");
        let dynamic_conf_path = settings.neard_home.join("dyn_config.json");
        assert!(metadata(&socket_path).is_err(), "previous socket unclose");
        let listener =
            UnixListener::bind(&socket_path).context("Failed to bind kuutamod control socket")?;
        let ctl_socket = UnixListenerStream::new(listener);
        Ok(Self {
            ctl_socket,
            current_state,
            dynamic_conf_path,
            near_rpc_port: settings.near_rpc_addr.port(),
            socket_path,
            neard_process_id,
        })
    }

    /// Excute and change StateType if needed
    pub async fn command_excutor(&self, cmd: &Command) -> Result<Option<StateType>> {
        match (self.current_state, cmd) {
            (StateType::Validating, Command::MaintenanceShutdown) => {
                let b = maintenance_shutdown::execute(
                    self.near_rpc_port,
                    self.neard_process_id,
                    &self.dynamic_conf_path,
                )
                .await?;
                info!("will maintenance shutdown at {b:?}");
                Ok(None)
            }
            (_, Command::MaintenanceShutdown) => {
                warn!("maintenance shutdown only accept in Validating state");
                Ok(None)
            }
            (_, Command::ActiveValidator) => panic!("Client side command should not be hear"),
        }
    }

    /// Read control socket and excute command, return new state if needed
    pub async fn listen(&mut self) -> Result<Option<StateType>> {
        if let Some(Ok(stream)) = self.ctl_socket.next().await {
            let ready = stream
                .ready(Interest::WRITABLE | Interest::READABLE)
                .await?;
            if ready.is_readable() {
                let mut data = vec![0; 1024];
                match stream.try_read(&mut data) {
                    Ok(_n) => {
                        let input = match std::str::from_utf8(&data) {
                            Ok(s) => s.trim_matches(char::from(0)),
                            Err(e) => {
                                return Err(e.into());
                            }
                        };
                        if let Ok(cmd) = serde_json::from_str::<Command>(input) {
                            match self.command_excutor(&cmd).await {
                                Ok(s) => Ok(s),
                                Err(e) => {
                                    bail!("excute cmd: {cmd:?} fail: {e:}");
                                }
                            }
                        } else {
                            bail!("unreadable cmd: {input:?}")
                        }
                    }
                    Err(e) => {
                        bail!("control socket can not read stream: {e:?}")
                    }
                }
            } else {
                bail!("control socket is not readable")
            }
        } else {
            Ok(None)
        }
    }
}

impl<'a> Drop for CommandHander<'a> {
    fn drop(&mut self) {
        for _ in 1..=3 {
            if metadata(&self.socket_path).is_ok() {
                if remove_file(&self.socket_path).is_ok() && metadata(&self.socket_path).is_err() {
                    break;
                }
            } else {
                break;
            }
        }
        if metadata(&self.socket_path).is_ok() || metadata(&self.dynamic_conf_path).is_ok() {
            panic!("fail to close control socket or dynamic config");
        }
    }
}

#[allow(dead_code)]
/// A client interact with kuutamo
#[derive(Debug)]
pub struct KuutamodClient {
    ctl_socket: UnixStream,
}

impl KuutamodClient {
    /// Returns a new Neard client for the given endpoint
    pub async fn new(socket_path: &PathBuf) -> Result<Self> {
        let ctl_socket = UnixStream::connect(socket_path).await?;
        Ok(Self { ctl_socket })
    }

    /// Send command
    pub async fn send(&self, cmd: Command) -> Result<()> {
        for i in 1..=3 {
            match self
                .ctl_socket
                .ready(Interest::WRITABLE | Interest::READABLE)
                .await
            {
                Ok(_ready) => {
                    match self.ctl_socket.try_write(
                        &serde_json::to_vec(&cmd).expect("all command shoud be serialized type"),
                    ) {
                        Ok(_) => {
                            return Ok(());
                        }
                        Err(e) => println!("{i} time try send command error {e:?}"),
                    }
                }
                Err(e) => eprintln!("{i} time send command error: {e:?}"),
            }

            println!("{i} time try excute cmd: {cmd:?}");
        }
        bail!("fail to send command")
    }
}
