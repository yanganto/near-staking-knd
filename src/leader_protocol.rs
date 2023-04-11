//! Interface used for leader election

/// Consul key used for leadership election
pub fn consul_leader_key(account_name: &str) -> String {
    format!("kuutamod-leader/{account_name}")
}
