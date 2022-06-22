//! Interface used for leader election

//pub const CONSUL_LEADER_KEY: &str = "kuutamod-leader";

/// Consul key used for leadership election
pub fn consul_leader_key(account_name: &str) -> String {
    return format!("kuutamod-leader/{}", account_name);
}
