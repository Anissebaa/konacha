use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn tmp_path(path: &Path) -> PathBuf {
    let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_nanos())
    .unwrap_or(0);
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("konacha");
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{name}.{nonce}.tmp"))
}

/// Durable atomic write: temp → fsync → rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(p) = path.parent() { fs::create_dir_all(p)?; }
    let tmp = tmp_path(path);
    {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

pub fn read(path: &Path) -> io::Result<Vec<u8>> { fs::read(path) }
