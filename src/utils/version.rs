use crate::deploy::Host;
use crate::utils::ssh::{ssh_with_timeout, ssh_with_timeout_async};
use anyhow::{anyhow, Context, Result};
use semver::{Version, VersionReq};
use std::process::Output;

/// async check the version of kuutamoctl meet requirement or not
pub async fn require_async(host: &Host, requirement: &str) -> Result<(bool, String)> {
    let Output { stdout, .. } =
        ssh_with_timeout_async(host, vec!["kuutamoctl".into(), "-V".into()], true, false)
            .await
            .context("Failed to fetch kuutamoctl version")?;
    let version_str =
        std::str::from_utf8(&stdout).map(|s| s.rsplit_once(' ').map(|(_, v)| v.trim()))?;
    let version =
        Version::parse(version_str.ok_or(anyhow!("version is not prefix with binary name"))?)
            .context("Failed to parse kuutamoctl version")?;

    Ok((
        VersionReq::parse(requirement)?.matches(&version),
        version_str.unwrap_or("unknown").into(),
    ))
}

/// check the version of kuutamoctl meet requirement or not
pub fn require(host: &Host, requirement: &str) -> Result<(bool, String)> {
    let Output { stdout, .. } = ssh_with_timeout(host, &["kuutamoctl", "-V"], true, false)
        .context("Failed to fetch kuutamoctl version")?;
    let version_str =
        std::str::from_utf8(&stdout).map(|s| s.rsplit_once(' ').map(|(_, v)| v.trim()))?;
    let version =
        Version::parse(version_str.ok_or(anyhow!("version is not prefix with binary name"))?)
            .context("Failed to parse kuutamoctl version")?;

    Ok((
        VersionReq::parse(requirement)?.matches(&version),
        version_str.unwrap_or("unknown").into(),
    ))
}
