use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::Metadata;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use jwalk::WalkDirGeneric;

pub struct Options {
    /// Group sizes by entries this many levels below the root (0 = no grouping).
    pub depth: usize,
    /// Keep this many of the largest files (0 = none).
    pub top_files: usize,
    /// Descend into other filesystems mounted under the root.
    pub cross_fs: bool,
}

#[derive(Debug, Default)]
pub struct Scan {
    /// Disk usage of everything under the root, in bytes.
    pub total: u64,
    /// Disk usage of each entry `depth` levels below the root, largest first.
    pub entries: Vec<(u64, PathBuf)>,
    /// Largest files, largest first.
    pub files: Vec<(u64, PathBuf)>,
    /// Paths that could not be read (permission denied, vanished, ...).
    pub errors: u64,
}

// Each entry carries the metadata fetched while filtering its parent directory,
// so it is only stat'ed once.
type Walk = WalkDirGeneric<((), Option<Metadata>)>;

pub fn scan(root: &Path, opts: &Options) -> io::Result<Scan> {
    let root_dev = std::fs::metadata(root)?.dev();
    let cross_fs = opts.cross_fs;

    let walk = Walk::new(root)
        .skip_hidden(false)
        .follow_links(false)
        .process_read_dir(move |_, _, _, children| {
            for child in children.iter_mut().flatten() {
                child.client_state = child.metadata().ok();
            }
            if !cross_fs {
                children.retain(|child| match child {
                    Ok(child) => child
                        .client_state
                        .as_ref()
                        .is_none_or(|m| m.dev() == root_dev),
                    Err(_) => true,
                });
            }
        });

    let mut scan = Scan::default();
    let mut entries: HashMap<PathBuf, u64> = HashMap::new();
    let mut files = BinaryHeap::new();
    let mut seen_links = HashSet::new();

    for entry in walk {
        let Ok(mut entry) = entry else {
            scan.errors += 1;
            continue;
        };
        if entry
            .read_children
            .as_ref()
            .is_some_and(|rc| rc.error().is_some())
        {
            scan.errors += 1;
        }
        let meta = match entry.client_state.take() {
            Some(meta) => meta,
            None => match entry.metadata() {
                Ok(meta) => meta,
                Err(_) => {
                    scan.errors += 1;
                    continue;
                }
            },
        };
        // Count hardlinked files once.
        if meta.nlink() > 1 && !meta.is_dir() && !seen_links.insert((meta.dev(), meta.ino())) {
            continue;
        }

        let size = meta.blocks() * 512;
        scan.total += size;

        if opts.depth > 0 && entry.depth > 0 {
            let path = entry.path();
            let up = entry.depth.saturating_sub(opts.depth);
            let key = path.ancestors().nth(up).unwrap_or(&path).to_path_buf();
            *entries.entry(key).or_default() += size;
        }

        if opts.top_files > 0 && meta.is_file() {
            let smallest = files.peek().map_or(0, |Reverse((s, _))| *s);
            if files.len() < opts.top_files || size > smallest {
                files.push(Reverse((size, entry.path())));
                if files.len() > opts.top_files {
                    files.pop();
                }
            }
        }
    }

    scan.entries = entries.into_iter().map(|(p, s)| (s, p)).collect();
    scan.entries.sort_by(|a, b| b.cmp(a));
    scan.files = files
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse(f)| f)
        .collect();
    Ok(scan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(path: &Path, kib: usize) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, vec![1u8; kib * 1024]).unwrap();
    }

    fn tree() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        write(&root.join("a/b/big"), 512);
        write(&root.join("a/small"), 16);
        write(&root.join("c/medium"), 128);
        write(&root.join("top"), 64);
        (dir, root)
    }

    fn opts(depth: usize, top_files: usize) -> Options {
        Options {
            depth,
            top_files,
            cross_fs: false,
        }
    }

    #[test]
    fn groups_by_depth() {
        let (_dir, root) = tree();

        let s = scan(&root, &opts(1, 0)).unwrap();
        let paths: Vec<_> = s.entries.iter().map(|(_, p)| p.clone()).collect();
        assert_eq!(paths, [root.join("a"), root.join("c"), root.join("top")]);
        assert_eq!(s.errors, 0);

        let s = scan(&root, &opts(2, 0)).unwrap();
        assert_eq!(s.entries[0].1, root.join("a/b"));
        assert!(s.entries.iter().any(|(_, p)| p == &root.join("a/small")));
    }

    #[test]
    fn top_files_largest_first() {
        let (_dir, root) = tree();
        let s = scan(&root, &opts(0, 2)).unwrap();
        let paths: Vec<_> = s.files.iter().map(|(_, p)| p.clone()).collect();
        assert_eq!(paths, [root.join("a/b/big"), root.join("c/medium")]);
        assert!(s.entries.is_empty());
    }

    #[test]
    fn hardlinks_counted_once() {
        let (_dir, root) = tree();
        let before = scan(&root, &opts(0, 10)).unwrap();
        fs::hard_link(root.join("a/b/big"), root.join("c/link")).unwrap();
        let after = scan(&root, &opts(0, 10)).unwrap();
        assert_eq!(before.files.len(), after.files.len());
        // Allow for the directory itself growing by a block.
        assert!(after.total - before.total < 64 * 1024);
    }
}
