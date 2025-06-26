# Rufus

A general-purpose tool for detecting cheating in autograded Gradescope code submissions by cross-referencing emissions of values that ought to be unique class-wide.

## Pre-built Binaries (Recommended)

You can download pre-built binaries for Linux, macOS, and Windows from the [GitHub Releases page](https://github.com/<your-username>/<your-repo>/releases).

- **Linux:** `rufus-linux`
- **macOS:** `rufus-macos`
- **Windows:** `rufus-windows.exe`

### Usage

1. Download the appropriate binary for your platform.
2. On Linux/macOS, make it executable:
   ```bash
   chmod +x rufus-<platform>
   ```
3. Run the binary from your terminal:
   ```bash
   ./rufus-<platform> [SUBCOMMAND] [ARGS]
   ```
4. For help and available commands:
   ```bash
   ./rufus-<platform> --help
   ```

See the [Releases page](https://github.com/UF-Comp-Linear-Algebra/Rufus/releases) for the latest version.

---

## Development

If you want to build or modify Rufus, follow these instructions:

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (install via [rustup.rs](https://www.rust-lang.org/tools/install))

### Building from Source

Clone the repository and build the project:

```bash
git clone https://github.com/<your-username>/<your-repo>.git
cd <your-repo>
cargo build --release
```

### Running from Source

You can run Rufus directly with Cargo:

```bash
cargo run --release -- [SUBCOMMAND] [ARGS]
```

For example, to hunt for cheating submissions:

```bash
cargo run --release -- hunt [FILES] --show-emissions
```

For help and available commands:

```bash
cargo run --release -- --help
```
