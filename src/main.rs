mod disk;
mod print;
mod scan;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use crate::disk::Disk;
use crate::scan::{Options, scan};

/// Find what's using disk space. Read-only: never deletes anything.
#[derive(Parser)]
#[command(version, args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Directory to inspect
    #[arg(default_value = "/")]
    path: PathBuf,

    /// Group sizes this many levels below the path
    #[arg(short, long, default_value_t = 1)]
    depth: usize,

    #[command(flatten)]
    common: Common,
}

#[derive(Subcommand)]
enum Command {
    /// List the largest files under a path
    Files {
        /// Directory to inspect
        #[arg(default_value = "/")]
        path: PathBuf,

        #[command(flatten)]
        common: Common,
    },
}

#[derive(Args)]
struct Common {
    /// Number of rows to show
    #[arg(short, default_value_t = 10)]
    n: usize,

    /// Descend into other filesystems (default: stay on one, like du -x)
    #[arg(short, long)]
    all: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        None => dirs(&cli.path, cli.depth, &cli.common),
        Some(Command::Files { path, common }) => files(&path, &common),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("hog: {err}");
            ExitCode::FAILURE
        }
    }
}

fn dirs(path: &Path, depth: usize, common: &Common) -> Result<(), String> {
    let root = resolve(path)?;
    let opts = Options {
        depth,
        top_files: 0,
        cross_fs: common.all,
    };
    let scan = scan(&root, &opts).map_err(|e| format!("{}: {e}", root.display()))?;

    print::disk(&Disk::of(&root).map_err(|e| format!("{}: {e}", root.display()))?);
    print::rows(&[(scan.total, root)]);
    print::rows(&scan.entries[..scan.entries.len().min(common.n)]);
    print::errors(scan.errors);
    Ok(())
}

fn files(path: &Path, common: &Common) -> Result<(), String> {
    let root = resolve(path)?;
    let opts = Options {
        depth: 0,
        top_files: common.n,
        cross_fs: common.all,
    };
    let scan = scan(&root, &opts).map_err(|e| format!("{}: {e}", root.display()))?;

    print::disk(&Disk::of(&root).map_err(|e| format!("{}: {e}", root.display()))?);
    print::rows(&scan.files);
    print::errors(scan.errors);
    Ok(())
}

fn resolve(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))
}
