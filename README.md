# Rufus

A general-purpose CLI for working with [Gradescope](https://www.gradescope.com) submission exports. It can detect plagiarism across autograded submissions, extract embedded file data, and walk submissions for manual grading.

## Pre-built Binaries (Recommended)

Download pre-built binaries for Linux, macOS, and Windows from the [GitHub Releases page](https://github.com/UF-Comp-Linear-Algebra/Rufus/releases).

- **Linux:** `rufus-linux`
- **macOS:** `rufus-macos`
- **Windows:** `rufus-windows.exe`

On Linux/macOS, make it executable before use:

```bash
chmod +x rufus-<platform>
./rufus-<platform> --help
```

---

## Commands

### `count`

Count the number of submissions across one or more export files.

```bash
rufus count submissions.yml
```

---

### `hunt`

Detect plagiarism by finding groups of submissions that share identical emissions.

```bash
rufus hunt submissions.yml [OPTIONS]
```

| Flag | Short | Description |
|------|-------|-------------|
| `--group-size N` | `-k` | Number of emissions that must match (default: max found) |
| `--min-size N` | `-m` | Minimum group size to display (default: 2) |
| `--show-emissions` | `-S` | Print the matching emissions for each group |
| `--exact` | `-E` | Only show groups matching exactly on k emissions |

```bash
# Find groups sharing any 3 emissions, show the matching values
rufus hunt submissions.yml -k 3 --show-emissions
```

---

### `extract`

Extract `extra_data` fields from Gradescope export files — as displayed text or written to files.

```bash
rufus extract submissions.yml [OPTIONS]
```

#### Key specification

Use `--key name[:decode[:ext]]` to select a field and optionally decode it:

| Example | Meaning |
|---------|---------|
| `--key level_uf2` | Raw value |
| `--key level_uf2:base64:uf2` | Base64-decode, save as `.uf2` |
| `--key checksum:hex` | Hex-decode |
| `--key checksum::sha256` | Raw value, save as `.sha256` |

`decode` can be `raw` (or empty), `base64`, `base64url`, or `hex`. Omit `--key` entirely to extract all available keys.

#### Output options

| Flag | Short | Description |
|------|-------|-------------|
| `--output DIR` | `-o` | Write files to DIR (default: print to stdout) |
| `--alongside` | `-A` | Write files into the same directory as each source YAML |
| `--layout` | | `dir` (subdirectory per submission, default) or `flat` (prefixed filenames) |
| `--name-by` | | `submission_id` (default), `name`, `email`, or `sid` |
| `--dry-run` | | Show paths that would be written without writing |

`--alongside` and `--output` are mutually exclusive.

#### Filtering options

| Flag | Description |
|------|-------------|
| `--list` | List all available keys and exit |
| `--skip-missing` | Skip submissions that don't have all selected keys |
| `--missing-only` | Report which submissions are missing selected keys |

```bash
# List available keys
rufus extract submissions.yml --list

# Extract a base64-encoded UF2 file for each submission into ./out/
rufus extract submissions.yml --key level_uf2:base64:uf2 -o ./out/

# Write files alongside the source YAML, skip submissions with missing keys
rufus extract submissions.yml --key level_uf2:base64:uf2 -A --skip-missing

# Preview what would be written
rufus extract submissions.yml --key level_uf2:base64:uf2 -o ./out/ --dry-run
```

---

### `grade`

Walk a directory of extracted submissions, open each one's Gradescope grading page in the browser, and track progress so you can resume later.

```bash
rufus grade <DIR> --course <ID> --assignment <ID> [OPTIONS]
```

| Flag | Short | Description |
|------|-------|-------------|
| `--course ID` | `-c` | Gradescope course ID |
| `--assignment ID` | `-a` | Gradescope assignment ID |
| `--export FILE` | `-e` | Export YAML to show submitter names |
| `--cmd TEMPLATE` | `-x` | Command to run once per submission |
| `--state FILE` | `-s` | State file path (default: `<DIR>/.rufus-grade`) |
| `--reset` | | Ignore existing state and start over |

The grader opens `https://www.gradescope.com/courses/{course}/assignments/{assignment}/submissions/{id}` for each submission directory named `submission_{id}`. Progress is saved after each submission so you can quit and resume with the same command (without `<DIR>`, `--course`, or `--assignment` — they are read from the state file).

#### Command templates

`--cmd` supports two placeholders:

| Placeholder | Expands to |
|-------------|-----------|
| `{dir}` | Absolute path to the submission directory |
| `{file:name}` | `{dir}/name` |

```bash
# Flash a UF2 file and open the grading page for each submission
rufus grade ./out/ -c 1217863 -a 7420607 \
  --export submissions.yml \
  --cmd "picotool load {file:level_uf2.uf2}"
```

#### Interactive prompt

```
[Enter] done  [s] skip  [r] re-run  [b] back  [q] quit
```

#### Typical workflow

```bash
# 1. Extract UF2 files from a Gradescope export
rufus extract submissions.yml --key level_uf2:base64:uf2 -o ./uf2s/

# 2. Grade each submission (flash hardware, open browser)
rufus grade ./uf2s/ -c 1217863 -a 7420607 \
  --export submissions.yml \
  --cmd "picotool load {file:level_uf2.uf2}"

# 3. Resume later — state is loaded automatically
rufus grade --state ./uf2s/.rufus-grade --export submissions.yml \
  --cmd "picotool load {file:level_uf2.uf2}"
```

---

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (install via `rustup`)

### Building

```bash
git clone https://github.com/UF-Comp-Linear-Algebra/Rufus.git
cd Rufus
cargo build --release
```

### Running

```bash
cargo run --release -- [SUBCOMMAND] [ARGS]
```

### Testing

```bash
cargo test
```
