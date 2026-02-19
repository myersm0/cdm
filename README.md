# cdm

[![CI](https://github.com/myersm0/cdm/actions/workflows/ci.yml/badge.svg)](https://github.com/myersm0/cdm/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/myersm0/cdm)](https://github.com/myersm0/cdm/releases/latest)

A collection of shell navigation commands with memory: jump to recent, frequent, or statistically associated directories. Fast `cd`-like filesystem navigation powered by your history.

This is a more robust, more feature-rich successor to my older repo with similar functionality, [bash-productivity](https://github.com/myersm0/bash-productivity), which was in part adapted from the book Pro Bash Programming by Chris F.A. Johnson.

## Commands

| Command | Description |
|---------|-------------|
| `goahead` | Pick from directories below cwd |
| `cdr` | Pick from most recently visited directories |
| `cdf` | Pick from most frequently visited directories |
| `cdp` | Pick from statistically co-accessed directories (NPMI) |

All commands present a numbered menu. Type a number and press Enter to jump there, or `q` to cancel.

## Usage

```bash
# directories ahead, 3 levels deep (default)
goahead

# deeper search, more results
goahead -d 5 -n 25

# recently visited
cdr

# recently visited, matching a pattern
cdr -r "my_project"

# recently visited, only under cwd
cdr -p

# frequently visited (within last 500 history entries by default)
cdf

# frequently visited, scoped to last 100 entries
cdf -H 100

# co-accessed: "where do I usually go from here?"
cdp
```

## How `cdp` works

`cdp` uses normalized pointwise mutual information (NPMI) to find directories that you tend to visit in the same session as your current directory.

For example, if you frequently switch between `~/my_projects/backend` and `~/my_projects/frontend`, running `cdp` from either one will suggest the other.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/myersm0/cdm/main/install.sh | sh
```

This detects your platform, downloads the latest precompiled binary, and installs it to `~/.local/bin` with shell functions in `~/.local/share/cdm/`. It then prints the lines to add to your shell profile.

### Shell setup (required)

Add to your `.bashrc`, `.zshrc`, etc.:

```bash
export PATH="$HOME/.local/bin:$PATH"  # if not already there
source /path/to/cdm/shell/cdm.sh
```

The `source` line loads a `cd` override that records history, plus the wrapper functions (`goahead`, `cdr`, `cdf`, `cdp`) that capture cdm's output and actually `cd` into the selected directory.

## Configuration

Optional. Create `~/.config/cdm/config.toml`:

```toml
# defaults shown
history_path = "~/.cd_history"
coaccess_window = 3

# shorten long paths in the picker display
[path_aliases]
proj = "/very/long/path/projects"
data = "/another/long/path/datasets"
```

With aliases configured, `/very/long/path/projects/foo` displays as `[proj]/foo`.

## Running tests

```bash
cargo test
```
