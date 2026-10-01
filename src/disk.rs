use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Usage of the filesystem containing a path, as `df` reports it.
pub struct Disk {
    pub mount: PathBuf,
    pub size: u64,
    pub used: u64,
    pub avail: u64,
}

impl Disk {
    pub fn of(path: &Path) -> io::Result<Self> {
        let st = rustix::fs::statvfs(path)?;
        let block = st.f_frsize;
        Ok(Self {
            mount: mount_point(path)?,
            size: st.f_blocks * block,
            used: (st.f_blocks - st.f_bfree) * block,
            avail: st.f_bavail * block,
        })
    }

    /// Percent used, rounded up, against space usable by non-root users (like `df`).
    pub fn percent(&self) -> u64 {
        let usable = self.used + self.avail;
        if usable == 0 {
            0
        } else {
            (self.used * 100).div_ceil(usable)
        }
    }
}

/// Walk up from `path` until the parent is on a different device.
fn mount_point(path: &Path) -> io::Result<PathBuf> {
    let dev = std::fs::metadata(path)?.dev();
    let mut mount = path;
    while let Some(parent) = mount.parent() {
        if std::fs::metadata(parent)?.dev() != dev {
            break;
        }
        mount = parent;
    }
    Ok(mount.to_path_buf())
}
