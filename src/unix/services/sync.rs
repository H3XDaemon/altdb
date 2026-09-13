//! Bounded, uncompressed ADB SYNC v1/v2, executed only after service identity drop.
use crate::protocol::{id, invalid};
use anyhow::{Result, bail, ensure};
use std::{
    ffi::{CString, OsString},
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    os::unix::{
        ffi::{OsStrExt, OsStringExt},
        fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
};

const DATA: u32 = id(b"DATA");
const DONE: u32 = id(b"DONE");
const MAX: usize = 65536;
fn number(r: &mut impl Read) -> io::Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn u32s(w: &mut impl Write, n: &[u32]) -> io::Result<()> {
    for v in n {
        w.write_all(&v.to_le_bytes())?;
    }
    Ok(())
}

fn fail(w: &mut impl Write, message: &str) -> Result<()> {
    let bytes = message.as_bytes();
    let bytes = &bytes[..bytes.len().min(256)];
    u32s(w, &[id(b"FAIL"), bytes.len() as u32])?;
    w.write_all(bytes)?;
    Ok(())
}

fn metadata(path: &Path, follow: bool) -> io::Result<Metadata> {
    if follow {
        fs::metadata(path)
    } else {
        fs::symlink_metadata(path)
    }
}

fn stat(
    w: &mut impl Write,
    kind: u32,
    result: io::Result<Metadata>,
    name: Option<&[u8]>,
    v2: bool,
) -> Result<()> {
    let (error, m) = match result {
        Ok(m) => (0, Some(m)),
        Err(e) => (e.raw_os_error().unwrap_or(libc::EIO) as u32, None),
    };
    if v2 {
        u32s(w, &[kind, error])?;
        for n in [
            m.as_ref().map_or(0, MetadataExt::dev),
            m.as_ref().map_or(0, MetadataExt::ino),
        ] {
            w.write_all(&n.to_le_bytes())?;
        }
        u32s(
            w,
            &[
                m.as_ref().map_or(0, MetadataExt::mode),
                m.as_ref().map_or(0, |m| m.nlink() as u32),
                m.as_ref().map_or(0, MetadataExt::uid),
                m.as_ref().map_or(0, MetadataExt::gid),
            ],
        )?;
        for n in [
            m.as_ref().map_or(0, MetadataExt::size),
            m.as_ref().map_or(0, |m| m.atime() as u64),
            m.as_ref().map_or(0, |m| m.mtime() as u64),
            m.as_ref().map_or(0, |m| m.ctime() as u64),
        ] {
            w.write_all(&n.to_le_bytes())?;
        }
    } else {
        u32s(
            w,
            &[
                kind,
                m.as_ref().map_or(0, MetadataExt::mode),
                m.as_ref().map_or(0, |m| m.size() as u32),
                m.as_ref().map_or(0, |m| m.mtime() as u32),
            ],
        )?;
    }
    if let Some(name) = name {
        u32s(w, &[name.len() as u32])?;
        w.write_all(name)?;
    }
    Ok(())
}

pub fn serve<S: Read + Write>(stream: &mut S) -> Result<()> {
    loop {
        let kind = number(stream)?;
        let len = number(stream)? as usize;
        if kind == id(b"QUIT") {
            ensure!(len == 0, "invalid QUIT");
            return Ok(());
        }
        if len == 0 || len > 1024 {
            fail(stream, "invalid path length")?;
            bail!("invalid path length")
        }
        let mut path = vec![0; len];
        stream.read_exact(&mut path)?;
        if path.contains(&0) {
            fail(stream, "invalid path")?;
            bail!("invalid path")
        }
        let result = handle(stream, kind, path);
        if let Err(e) = result {
            let message = if let Some(io) = e.downcast_ref::<io::Error>() {
                format!(
                    "filesystem error: {}",
                    io.raw_os_error().unwrap_or(libc::EIO)
                )
            } else {
                "unsupported or malformed SYNC request".into()
            };
            let _ = fail(stream, &message);
            return Err(e);
        }
    }
}

fn handle<S: Read + Write>(stream: &mut S, kind: u32, raw: Vec<u8>) -> Result<()> {
    let path = PathBuf::from(OsString::from_vec(raw.clone()));
    match kind {
        n if n == id(b"STAT") => stat(stream, kind, metadata(&path, false), None, false),
        n if n == id(b"STA2") || n == id(b"LST2") => {
            stat(stream, kind, metadata(&path, n == id(b"STA2")), None, true)
        }
        n if n == id(b"LIST") || n == id(b"LIS2") => {
            let v2 = n == id(b"LIS2");
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries {
                    let entry = entry?;
                    let name = entry.file_name();
                    stat(
                        stream,
                        if v2 { id(b"DNT2") } else { id(b"DENT") },
                        fs::symlink_metadata(entry.path()),
                        Some(name.as_bytes()),
                        v2,
                    )?;
                }
            }
            stat(
                stream,
                DONE,
                Err(io::Error::from_raw_os_error(0)),
                Some(&[]),
                v2,
            )
        }
        n if n == id(b"RECV") || n == id(b"RCV2") => {
            if n == id(b"RCV2") {
                ensure!(
                    number(stream)? == n && number(stream)? == 0,
                    "unsupported receive flags"
                );
            }
            let mut file = File::open(&path)?;
            let mut buf = vec![0; MAX];
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                u32s(stream, &[DATA, n as u32])?;
                stream.write_all(&buf[..n])?;
            }
            u32s(stream, &[DONE, 0])?;
            Ok(())
        }
        n if n == id(b"SEND") || n == id(b"SND2") => {
            let (path, mode) = if n == id(b"SEND") {
                let comma = raw
                    .iter()
                    .rposition(|b| *b == b',')
                    .ok_or_else(|| invalid("missing file mode"))?;
                let mode = std::str::from_utf8(&raw[comma + 1..])?.parse::<u32>()?;
                (
                    PathBuf::from(OsString::from_vec(raw[..comma].to_vec())),
                    mode,
                )
            } else {
                ensure!(number(stream)? == n, "invalid SEND setup");
                let mode = number(stream)?;
                ensure!(number(stream)? == 0, "unsupported send flags");
                (path, mode)
            };
            send_file(stream, &path, mode)
        }
        _ => bail!("unsupported SYNC operation"),
    }
}

fn send_file<S: Read + Write>(stream: &mut S, path: &Path, mode: u32) -> Result<()> {
    ensure!(path.file_name().is_some(), "invalid target");
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".altdb-push-{}",
        crate::crypto::hex(&crate::crypto::random::<8>()?)
    ));
    let result = (|| -> Result<()> {
        let symlink = mode & libc::S_IFMT == libc::S_IFLNK;
        ensure!(
            mode & libc::S_IFMT == 0 || mode & libc::S_IFMT == libc::S_IFREG || symlink,
            "unsupported file type"
        );
        let mut file = if symlink {
            None
        } else {
            Some(
                OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .mode(0o600)
                    .custom_flags(libc::O_NOFOLLOW)
                    .open(&temp)?,
            )
        };
        let mut target = vec![];
        let mtime = loop {
            let kind = number(stream)?;
            let len = number(stream)?;
            if kind == DONE {
                break len;
            }
            ensure!(
                kind == DATA && len as usize <= MAX,
                "invalid file data frame"
            );
            let mut buf = vec![0; len as usize];
            stream.read_exact(&mut buf)?;
            if let Some(file) = &mut file {
                file.write_all(&buf)?;
            } else {
                ensure!(target.len() + buf.len() <= 4096, "symlink target too long");
                target.extend(buf);
            }
        };
        if let Some(file) = file {
            file.set_permissions(fs::Permissions::from_mode(mode & 0o777))?;
            file.sync_all()?;
        } else {
            if target.last() == Some(&0) {
                target.pop();
            }
            ensure!(
                !target.is_empty() && !target.contains(&0),
                "invalid symlink target"
            );
            std::os::unix::fs::symlink(OsString::from_vec(target), &temp)?;
        }
        let c = CString::new(temp.as_os_str().as_bytes())?;
        let times = [libc::timespec {
            tv_sec: mtime as _,
            tv_nsec: 0,
        }; 2];
        ensure!(
            unsafe {
                libc::utimensat(
                    libc::AT_FDCWD,
                    c.as_ptr(),
                    times.as_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } == 0,
            "timestamp failed"
        );
        fs::rename(&temp, path)?;
        u32s(stream, &[id(b"OKAY"), 0])?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stat_v2_size() {
        let mut b = vec![];
        stat(
            &mut b,
            id(b"STA2"),
            Err(io::Error::from_raw_os_error(libc::ENOENT)),
            None,
            true,
        )
        .unwrap();
        assert_eq!(b.len(), 72);
        assert_eq!(
            u32::from_le_bytes(b[4..8].try_into().unwrap()),
            libc::ENOENT as u32
        );
    }
    #[test]
    fn list_done_sizes() {
        for (v2, size) in [(false, 20), (true, 76)] {
            let mut b = vec![];
            stat(
                &mut b,
                DONE,
                Err(io::Error::from_raw_os_error(0)),
                Some(&[]),
                v2,
            )
            .unwrap();
            assert_eq!(b.len(), size);
        }
    }
}
