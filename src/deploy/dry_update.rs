use anyhow::Result;

use super::Host;

/// Push update to server but do not activate it yet.
pub fn dry_update(_host: &[Host]) -> Result<()> {
    unimplemented!();
}
