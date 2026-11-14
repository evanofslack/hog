# hog

Find what's hogging your disk, read only.

```
$ hog ~
/  926.3 GiB  █████████░░░░░░░░░░░  45%  516.2 GiB free

 210.4 GiB  /Users/you
  98.1 GiB  /Users/you/Library
  61.7 GiB  /Users/you/projects
  22.3 GiB  /Users/you/Downloads
  ...
```

## Install

Linux (x86_64 / aarch64), no Rust needed:

```sh
curl -fsSL https://raw.githubusercontent.com/evanofslack/hog/main/install.sh | sh
```

Installs to `/usr/local/bin` if writable, otherwise `~/.local/bin`. Prebuilt binaries are also on the [releases](https://github.com/evanofslack/hog/releases) page.

From source (Rust 1.85+):

```sh
cargo install --git https://github.com/evanofslack/hog
```

## Usage

```sh
hog [PATH]          # biggest entries under PATH (default: /)
hog files [PATH]    # biggest individual files under PATH
```

| Flag | |
|---|---|
| `-n N` | rows to show (default 10) |
| `-d N` | group N levels deep (default 1, `hog` only) |
| `-a` | cross into other mounted filesystems |

## Examples

```sh
hog ~                       # what's big in my home dir?
hog ~/Library -d 2          # drill two levels deep
hog files ~/Downloads -n 20 # 20 largest files to clean up
sudo hog /                  # whole disk, including protected dirs
```

Sizes are actual disk usage, and hardlinks are counted once. Unreadable paths are skipped with a warning. Works on macOS and Linux.

## License

MIT
