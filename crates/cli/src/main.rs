use std::fs;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

const SOCKET: &str = "/tmp/sunlight.sock";

/// Sends scripts to sunlight.
#[derive(Parser)]
#[command(version)]
enum Cli {
    /// Queues a script to run once a game has loaded
    Exec { script: PathBuf },
}

fn main() -> Result<()> {
    let Cli::Exec { script } = Cli::parse();
    let source = fs::read(&script)
        .with_context(|| format!("failed to read {}", script.display()))?;
    UnixStream::connect(SOCKET)
        .with_context(|| format!("sunlight is not listening on {SOCKET}"))?
        .write_all(&source)?;
    Ok(())
}
