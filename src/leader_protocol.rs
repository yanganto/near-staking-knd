//! Interface used for leader election

//pub const CONSUL_LEADER_KEY: &str = "kneard-leader";

/// Consul key used for leadership election
pub fn consul_leader_key(account_name: &str) -> String {
    format!("kneard-leader/{account_name}")
}
