# cdm
`cd` with memory. Fast filesystem navigation powered by your history.

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
git clone https://github.com/youruser/cdm
cd cdm
cargo build --release
```

Add to your `.bashrc` or `.bash_profile`:

```bash
export PATH="/path/to/cdm/target/release:$PATH"
source /path/to/cdm/shell/cdm.sh
```

The `source` line is essential: it overrides `cd` to record history and defines the wrapper functions that capture cdm's output and actually `cd` into the selected directory.

## Configuration

Optional. Create `~/.config/cdm/config.toml`:

```toml
# defaults shown
history_path = "~/.cd_history"
coaccess_window = 3

# shorten long paths in the picker display
[path_aliases]
proj = "/very/long/corporate/path/projects"
data = "/another/long/path/datasets"
```

With aliases configured, `/very/long/path/projects/foo` displays as `[proj]/foo`.

