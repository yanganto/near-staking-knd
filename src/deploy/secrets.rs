use std::fs::{self, File};
use std::io::Read;
use std::{path::Path, process::Command};

use anyhow::{bail, Context, Result};
use tempfile::{Builder, TempDir};

use crate::deploy::command::status_to_pretty_err;

pub struct Secrets {
    tmp_dir: TempDir,
}

impl Secrets {
    pub fn new<'a, I>(secrets: I) -> Result<Self>
    where
        I: Iterator<Item = &'a (&'a Path, &'a Path)>,
    {
        let tmp_dir = Builder::new()
            .prefix("kuutamo-secrets.")
            .tempdir()
            .context("cannot create temporary directory")?;

        for (to, from) in secrets {
            let secret_path = tmp_dir.path().join(to.strip_prefix("/").unwrap_or(to));
            let dir = secret_path.parent().with_context(|| {
                format!("Cannot get parent of directory: {}", secret_path.display())
            })?;
            fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
            let mut content = Vec::new();
            // read the whole file
            let mut f =
                File::open(from).with_context(|| format!("cannot open {}", from.display()))?;
            f.read_to_end(&mut content)
                .with_context(|| format!("failed to read secret: {}", from.display()))?;

            fs::write(&secret_path, content).with_context(|| {
                format!(
                    "cannot write secret to temporary location at {}",
                    secret_path.display()
                )
            })?;
        }
        Ok(Self { tmp_dir })
    }
    /// Path to the nixos flake
    pub fn path(&self) -> &Path {
        self.tmp_dir.path()
    }

    // rsync -vrlF -e "ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no" "$extra_files" "${ssh_connection}:/mnt/"
    pub fn upload(&self, ssh_target: &str) -> Result<()> {
        // Do proper logging here?
        println!("Upload secrets");
        let path = self
            .path()
            .to_str()
            .context("Cannot convert secrets directory to string")?;
        let args = vec!["-vrlF", path, ssh_target];
        let status = Command::new("rsync").args(&args).status();
        status_to_pretty_err(status, "rsync", &args)?;
        Ok(())
    }
}
