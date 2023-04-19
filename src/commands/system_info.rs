use anyhow::Result;
use std::env;

/// Collect and print out system info
pub fn system_info(inline: bool) -> Result<()> {
    let info = vec![
        ("version", env!("CARGO_PKG_VERSION")),
        ("branch", env!("VERGEN_GIT_BRANCH")),
        ("hash", env!("VERGEN_GIT_DESCRIBE")),
        ("commit-date", env!("VERGEN_GIT_COMMIT_DATE")),
    ];

    if inline {
        let system_info: Vec<String> = info.iter().map(|i| format!("{}={}", i.0, i.1)).collect();
        println!("{}", system_info.join(" "))
    } else {
        let system_info: Vec<String> = info.iter().map(|i| format!("{}: {}", i.0, i.1)).collect();
        println!("{}", system_info.join("\n"))
    }
    Ok(())
}
