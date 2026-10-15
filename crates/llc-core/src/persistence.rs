use crate::*;
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub fn config_path() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok_or_else(|| Error::Invalid("HOME or absolute XDG_CONFIG_HOME is required".into()))?;
    Ok(base.join("linux-logitech-control/config.json"))
}
pub fn read(path: &Path) -> Result<Config> {
    let mut bytes = vec![];
    File::open(path)?
        .take((MAX_CONFIG_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    Config::parse(&bytes)
}
/// Recovery never modifies a corrupt primary or backup.
pub fn recover(path: &Path) -> Result<(Config, bool)> {
    match read(path) {
        Ok(config) => Ok((config, false)),
        Err(primary) => match read(&path.with_extension("json.bak")) {
            Ok(backup) => Ok((backup, true)),
            Err(_) => Err(primary),
        },
    }
}
fn private_open(path: &Path, create_new: bool) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).read(true);
    if create_new {
        options.create_new(true);
    } else {
        options.create(true).truncate(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc_nofollow());
    }
    options.open(path)
}
#[cfg(unix)]
fn libc_nofollow() -> i32 {
    0x20000
} // Linux O_NOFOLLOW; supported build target.
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let temp = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = private_open(&temp, true)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        #[cfg(unix)]
        File::open(path.parent().unwrap())?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
pub fn save(path: &Path, config: &Config, expected_revision: u64) -> Result<Config> {
    config.validate()?;
    let directory = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| Error::Invalid("save path must include parent directory".into()))?;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(directory)?;
    let lock = private_open(&path.with_extension("json.lock"), false)?;
    lock.lock_exclusive()?;
    let old = if path.exists() {
        Some(read(path)?)
    } else {
        None
    };
    let revision = old.as_ref().map_or(0, |c| c.revision);
    if revision != expected_revision {
        return Err(Error::Revision(revision));
    }
    let mut next = config.clone();
    next.revision = revision
        .checked_add(1)
        .ok_or_else(|| Error::Invalid("revision overflow".into()))?;
    let bytes = serde_json::to_vec_pretty(&next)?;
    if bytes.len() > MAX_CONFIG_BYTES {
        return invalid("configuration exceeds 4 MiB");
    }
    if let Some(old) = old {
        atomic_write(
            &path.with_extension("json.bak"),
            &serde_json::to_vec_pretty(&old)?,
        )?;
    }
    atomic_write(path, &bytes)?;
    Ok(next)
}

pub fn export(path: &Path, config: &Config) -> Result<()> {
    config.validate()?;
    let mut file = private_open(path, true)?;
    file.write_all(&serde_json::to_vec_pretty(&config.redacted_export())?)?;
    file.sync_all()?;
    Ok(())
}
