use anyhow::{bail, Result};
use regex::Regex;
use std::env;

fn parse_neard_version(raw_version: &str) -> Result<(String, String, String)> {
    let mut develop_version = false;
    let mut version = String::new();
    let mut protocol_version = String::new();
    let mut db_version = String::new();
    for cap in Regex::new(r#"\((?P<para>\S*)\s(?P<value>\S*)\)"#)?.captures_iter(raw_version) {
        if cap["para"] == *"release" && cap["value"] == *"trunk" {
            develop_version = true;
        }
        if cap["para"] == *"build" {
            version = cap["value"].to_string();
        }
        if cap["para"] == *"protocol" {
            protocol_version = cap["value"].to_string();
        }
        if cap["para"] == *"db" {
            db_version = cap["value"].to_string();
        }
    }
    if develop_version {
        Ok(("develop".into(), protocol_version, db_version))
    } else {
        Ok((version, protocol_version, db_version))
    }
}

fn neard_versions() -> Result<(String, String, String)> {
    let output = std::process::Command::new("neard").args(["-V"]).output()?;
    if output.status.success() {
        let output = std::str::from_utf8(&output.stdout)?;
        parse_neard_version(output)
    } else {
        bail!("fail to get version information from neard")
    }
}

/// Collect and print out system info
pub fn system_info(inline: bool) {
    let info = if let Ok((neard_version, protocol_version, db_version)) = neard_versions() {
        vec![
            ("kneard-version", env!("CARGO_PKG_VERSION").into()),
            ("git-sha", env!("VERGEN_GIT_SHA").into()),
            ("git-commit-date", env!("VERGEN_GIT_COMMIT_DATE").into()),
            ("neard-version", neard_version),
            ("neard-protocol-version", protocol_version),
            ("neard-db-version", db_version),
        ]
    } else {
        vec![
            ("keanrd-version", env!("CARGO_PKG_VERSION").into()),
            ("git-sha", env!("VERGEN_GIT_SHA").into()),
            ("git-commit-date", env!("VERGEN_GIT_COMMIT_DATE").into()),
        ]
    };

    if inline {
        let system_info: Vec<String> = info.iter().map(|i| format!("{}={}", i.0, i.1)).collect();
        println!("{}", system_info.join(" "))
    } else {
        let system_info: Vec<String> = info.iter().map(|i| format!("{}: {}", i.0, i.1)).collect();
        println!("{}", system_info.join("\n"))
    }
}

#[test]
fn test_parse_near_release_version_string() {
    assert_eq!(
        parse_neard_version("neard (release 1.32.2) (build 1.32.2-1-gbb5fd9436-modified) (rustc 1.68.0) (protocol 59) (db 34)").unwrap(),
        ("1.32.2-1-gbb5fd9436-modified".to_string(), "59".to_string(), "34".to_string())
    );
}

#[test]
fn test_parse_near_dev_version_string() {
    assert_eq!(
        parse_neard_version("neard (release trunk) (build 1.1.0-3557-g8a9acc0df) (rustc 1.68.0) (protocol 59) (db 35)").unwrap(),
        ("develop".to_string(), "59".to_string(), "35".to_string())
    );
}
