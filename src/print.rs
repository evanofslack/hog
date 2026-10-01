use std::path::PathBuf;

use humansize::{BINARY, FormatSizeOptions, format_size};

use crate::disk::Disk;

const BAR_WIDTH: u64 = 20;

fn size(bytes: u64) -> String {
    format_size(
        bytes,
        FormatSizeOptions::from(BINARY)
            .decimal_places(1)
            .decimal_zeroes(1),
    )
}

pub fn disk(d: &Disk) {
    let filled = (d.percent() * BAR_WIDTH).div_ceil(100).min(BAR_WIDTH) as usize;
    let bar = "█".repeat(filled) + &"░".repeat(BAR_WIDTH as usize - filled);
    println!(
        "{}  {}  {bar}  {}%  {} free\n",
        d.mount.display(),
        size(d.size),
        d.percent(),
        size(d.avail),
    );
}

pub fn rows(rows: &[(u64, PathBuf)]) {
    for (bytes, path) in rows {
        println!("{:>10}  {}", size(*bytes), path.display());
    }
}

pub fn errors(count: u64) {
    if count > 0 {
        eprintln!("\n⚠ {count} paths unreadable, sizes may be low (try sudo)");
    }
}
