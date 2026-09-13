use crate::{
    config::{Config, Host, MAX_HOSTS},
    crypto::{self, Identity},
};
use anyhow::{Result, ensure};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::{
        fs::{OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
    },
    path::{Path, PathBuf},
};

pub struct Store {
    pub path: PathBuf,
    pub config: Config,
    pub identity: Identity,
    pub hosts: Vec<Host>,
    _lock: File,
}
#[derive(Debug)]
pub struct AlreadyRunning;
impl std::fmt::Display for AlreadyRunning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("altdb is already running")
    }
}

impl std::error::Error for AlreadyRunning {}
pub fn atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing state directory"))?;
    let temp = parent.join(format!(".{}.tmp", crypto::hex(&crypto::random::<8>()?)));
    let backup = parent.join(format!(
        ".{}.rollback",
        crypto::hex(&crypto::random::<8>()?)
    ));
    let mut preserve_backup = false;
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&temp)?;
        serde_json::to_writer(&mut file, value)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        let directory = File::open(parent)?;
        let had_previous = match fs::hard_link(path, &backup) {
            Ok(()) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(e.into()),
        };
        directory.sync_all()?;
        fs::rename(&temp, path)?;
        if let Err(error) = directory.sync_all() {
            // Preserve the previously active configuration if the directory
            // cannot durably commit the rename. A rollback failure is explicit.
            let rollback = if had_previous {
                fs::rename(&backup, path)
            } else {
                fs::remove_file(path)
            };
            if rollback.is_err() {
                preserve_backup = true;
                anyhow::bail!("state commit and rollback failed; inspect storage before restart");
            }
            let _ = directory.sync_all();
            return Err(error.into());
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    if !preserve_backup {
        let _ = fs::remove_file(&backup);
    }
    result
}

fn read<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    ensure!(
        file.metadata()?.len() <= 1024 * 1024,
        "state file exceeds limit"
    );
    Ok(serde_json::from_reader(file)?)
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        ensure!(
            !fs::symlink_metadata(path)?.file_type().is_symlink(),
            "state directory must not be a symlink"
        );
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path.join("daemon.lock"))?;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::WouldBlock {
                return Err(AlreadyRunning.into());
            }
            return Err(error.into());
        }
        let config = if path.join("config.json").exists() {
            read::<Config>(&path.join("config.json"))?
        } else {
            let c = Config::default();
            atomic(&path.join("config.json"), &c)?;
            c
        };
        config.validate()?;
        let identity = if path.join("identity.json").exists() {
            read::<Identity>(&path.join("identity.json"))?
        } else {
            let i = Identity::generate()?;
            atomic(&path.join("identity.json"), &i)?;
            i
        };
        let hosts = if path.join("hosts.json").exists() {
            read::<Vec<Host>>(&path.join("hosts.json"))?
        } else {
            vec![]
        };
        ensure!(hosts.len() <= MAX_HOSTS, "too many stored hosts");
        for host in &hosts {
            ensure!(
                crypto::host_key(&host.public_key)?.0 == host.fingerprint,
                "host fingerprint mismatch"
            );
        }
        Ok(Self {
            path: path.to_owned(),
            config,
            identity,
            hosts,
            _lock: lock,
        })
    }

    pub fn configure(&mut self, config: Config) -> Result<()> {
        config.validate()?;
        atomic(&self.path.join("config.json"), &config)?;
        self.config = config;
        Ok(())
    }

    pub fn pair(&mut self, key: String) -> Result<String> {
        let (fp, name) = crypto::host_key(&key)?;
        // PeerInfo comments can consume almost 8 KiB. Persist only the actual
        // RSA key and the separately bounded display name, keeping bootstrap IPC
        // bounded even with the maximum number of paired hosts.
        let key = key.split_whitespace().next().unwrap_or_default().to_owned();
        let mut hosts = self.hosts.clone();
        if !hosts.iter().any(|h| h.fingerprint == fp) {
            ensure!(hosts.len() < MAX_HOSTS, "paired host limit reached");
            hosts.push(Host {
                fingerprint: fp.clone(),
                public_key: key,
                name,
                paired_at: super::now(),
                last_connected: None,
            });
        }
        atomic(&self.path.join("hosts.json"), &hosts)?;
        self.hosts = hosts;
        Ok(fp)
    }

    pub fn revoke(&mut self, fp: &str) -> Result<()> {
        let hosts: Vec<_> = self
            .hosts
            .iter()
            .filter(|h| h.fingerprint != fp)
            .cloned()
            .collect();
        atomic(&self.path.join("hosts.json"), &hosts)?;
        self.hosts = hosts;
        Ok(())
    }

    pub fn connected(&mut self, fp: &str) -> Result<()> {
        let mut hosts = self.hosts.clone();
        let host = hosts
            .iter_mut()
            .find(|h| h.fingerprint == fp)
            .ok_or_else(|| anyhow::anyhow!("host is not paired"))?;
        host.last_connected = Some(super::now());
        atomic(&self.path.join("hosts.json"), &hosts)?;
        self.hosts = hosts;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_save_keeps_active_config() {
        let d = tempfile::tempdir().unwrap();
        let mut s = Store::open(d.path()).unwrap();
        let mut wanted = s.config.clone();
        wanted.allow_adb_root = true;
        s.path = d.path().join("identity.json"); // A regular file cannot be a state directory.
        assert!(s.configure(wanted).is_err());
        assert!(!s.config.allow_adb_root);
        assert!(
            !read::<Config>(&d.path().join("config.json"))
                .unwrap()
                .allow_adb_root
        );
    }
    #[test]
    fn host_comments_do_not_overflow_bootstrap() {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let d = tempfile::tempdir().unwrap();
        let mut s = Store::open(d.path()).unwrap();
        let mut raw = vec![0u8; 524];
        raw[0] = 64;
        raw[8..264].fill(0xff);
        raw[520..524].copy_from_slice(&65537u32.to_le_bytes());
        let key = STANDARD.encode(raw);
        s.pair(format!("{key} {}", "测".repeat(2000))).unwrap();
        assert_eq!(s.hosts[0].public_key, key);
        assert_eq!(s.hosts[0].name.chars().count(), 128);
        drop(s);
        assert_eq!(Store::open(d.path()).unwrap().hosts.len(), 1);
    }
    #[test]
    fn state_is_private_and_updates_are_atomic() {
        let d = tempfile::tempdir().unwrap();
        let mut s = Store::open(d.path()).unwrap();
        let guid = s.identity.guid.clone();
        assert!(Store::open(d.path()).is_err());
        let mut c = s.config.clone();
        c.fixed_port = 1;
        assert!(s.configure(c).is_err());
        assert_eq!(s.config.fixed_port, 5555);
        assert_eq!(
            fs::metadata(d.path().join("identity.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        drop(s);
        assert_eq!(Store::open(d.path()).unwrap().identity.guid, guid);
    }
}
