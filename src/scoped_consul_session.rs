//! A consul session that can be manually deleted

use crate::consul_client::{ConsulClient, ConsulSession};
use log::warn;
use std::borrow::Borrow;
use std::time::Duration;
use std::time::Instant;

/// A wrapper around ConsulSession that also deletes the underlying consul session
/// The user needs to manually call `destroy` to destroy the session
pub struct ScopedConsulSession<'a> {
    inner: ConsulSession,
    client: &'a ConsulClient,
}

impl<'a> ScopedConsulSession<'a> {
    /// Returns a new ScopedConsulSession
    pub fn new(c: &ConsulClient, s: ConsulSession) -> ScopedConsulSession {
        ScopedConsulSession {
            inner: s,
            client: c,
        }
    }

    /// Consums and deletes consul session,
    /// We used to have this in `Drop` but we cannot do async stuff in their easily
    pub async fn destroy(self) {
        let mut wait = 1;
        let total_wait = Instant::now();
        // 1 + 2 + 4 + 5 + 5 + 5 + 5 == ~27s
        for _ in 0..7 {
            let res = self.client.delete_session(&self.inner).await;
            let e = match res {
                Ok(()) => return,
                Err(e) => e,
            };
            warn!(
                "Failed to deregister consul session {}: {} (wait: {}, elapsed: {})",
                self.inner.id(),
                e,
                wait,
                total_wait.elapsed().as_secs()
            );
            tokio::time::sleep(Duration::from_millis(wait)).await;
            // expontential backoff, capped at 5s
            wait = std::cmp::max(wait * 2, 5000);
        }
    }
}

impl Borrow<ConsulSession> for ScopedConsulSession<'_> {
    /// Returns a reference to the actual
    fn borrow(&self) -> &ConsulSession {
        &self.inner
    }
}

impl From<ScopedConsulSession<'_>> for ConsulSession {
    fn from(s: ScopedConsulSession) -> ConsulSession {
        s.inner
    }
}
