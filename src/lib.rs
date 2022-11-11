#![deny(missing_docs)]

//! a HA supervisor library for neard

pub mod commands;
pub mod consul_client;
pub mod exit_signal_handler;
pub mod ipc;
pub mod leader_protocol;
pub mod log_fmt;
pub mod near_client;
pub mod near_config;
pub mod neard_process;
pub mod oom_score;
pub mod proc;
pub mod prometheus;
pub mod scoped_consul_session;
pub mod settings;
pub mod supervisor;
