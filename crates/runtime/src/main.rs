//! Converts an iOS app to Mac Catalyst and launches it with an injected dylib.

use std::fs::{self, File, Permissions};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, io};

use anyhow::{Context, Result, bail};

const MACH_O_MAGIC: [u8; 4] = [0xcf, 0xfa, 0xed, 0xfe];
const FAT_MAGIC: [u8; 4] = [0xca, 0xfe, 0xba, 0xbe];

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let (Some(ipa), Some(dylib), None) =
        (args.next(), args.next(), args.next())
    else {
        bail!("usage: runtime <ipa> <dylib>");
    };
    let ipa = fs::canonicalize(&ipa)
        .with_context(|| format!("ipa not found at {}", ipa.display()))?;
    if ipa.extension().is_none_or(|extension| extension != "ipa") {
        bail!("{} is not an ipa", ipa.display());
    }
    let dylib = fs::canonicalize(&dylib)
        .with_context(|| format!("dylib not found at {}", dylib.display()))?;

    let app = install(&ipa)?;
    // shared across ipa versions so data like the login survives updates
    let home = ipa.with_file_name("home");
    fs::create_dir_all(&home)?;
    let log = ipa.with_extension("log");

    run(Command::new("open")
        .arg("--env")
        .arg(format!("DYLD_INSERT_LIBRARIES={}", dylib.display()))
        .arg("--env")
        .arg(format!("CFFIXED_USER_HOME={}", home.display()))
        .arg("--stdout")
        .arg(&log)
        .arg("--stderr")
        .arg(&log)
        .arg(&app))?;
    println!("logging to {}", log.display());
    Ok(())
}

fn install(ipa: &Path) -> Result<PathBuf> {
    let root = ipa.with_extension("");
    if !root.exists() {
        let partial = ipa.with_extension("partial");
        if partial.exists() {
            fs::remove_dir_all(&partial)?;
        }
        run(Command::new("ditto").args(["-x", "-k"]).arg(ipa).arg(&partial))?;
        convert(&find_app(&partial)?)?;
        fs::rename(&partial, &root)?;
    }
    find_app(&root)
}

fn find_app(root: &Path) -> Result<PathBuf> {
    let payload = root.join("Payload");
    let entries = fs::read_dir(&payload)
        .with_context(|| format!("no payload in {}", root.display()))?;
    for entry in entries {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "app") {
            return Ok(path);
        }
    }
    bail!("no app bundle in {}", payload.display())
}

fn convert(app: &Path) -> Result<()> {
    let plugins = app.join("PlugIns");
    if plugins.exists() {
        fs::remove_dir_all(plugins)?;
    }

    let mut binaries = Vec::new();
    find_binaries(app, &mut binaries)?;
    for binary in binaries {
        if magic(&binary)? == FAT_MAGIC {
            run(Command::new("lipo")
                .args(["-thin", "arm64", "-output"])
                .args([&binary, &binary]))?;
        }
        run(Command::new("vtool")
            .args([
                "-set-build-version",
                "maccatalyst",
                "11.0",
                "14.0",
                "-replace",
                "-output",
            ])
            .args([&binary, &binary]))?;
        // the ipa ships its binaries without the executable bit
        fs::set_permissions(&binary, Permissions::from_mode(0o755))?;
        // --deep skips the nested frameworks, so each binary is signed on its own
        run(Command::new("codesign")
            .args(["--force", "--sign", "-"])
            .arg(&binary))?;
    }

    run(Command::new("codesign").args(["--force", "--sign", "-"]).arg(app))?;
    Ok(())
}

fn find_binaries(directory: &Path, binaries: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            find_binaries(&path, binaries)?;
        } else if file_type.is_file()
            && [MACH_O_MAGIC, FAT_MAGIC].contains(&magic(&path)?)
        {
            binaries.push(path);
        }
    }
    Ok(())
}

fn magic(path: &Path) -> io::Result<[u8; 4]> {
    let mut magic = [0; 4];
    match File::open(path)?.read_exact(&mut magic) {
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
            Ok([0; 4])
        }
        result => result.map(|()| magic),
    }
}

fn run(command: &mut Command) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("failed to spawn {command:?}"))?;
    if !output.status.success() {
        bail!(
            "{command:?} failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}
