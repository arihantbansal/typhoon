# Typhoon CLI

A command-line interface for building Solana programs with the Typhoon framework.

## Installation

Install from crates.io:

```bash
cargo install typhoon-cli
```

Build from source:

```bash
cargo install --path cli
```

## Quick Start

```bash
# Create a new program
typhoon init my-program
cd my-program

# Build and test
typhoon build
typhoon test
```

## Usage

### Creating a New Program

Initialize a new Typhoon program using one of two templates:

```bash
typhoon init <name> [--template <template>]
```

The `counter` template (default) provides a full example with state management, multiple instructions, and IDL generation. The `hello-world` template offers a minimal single-instruction example with no state.

```bash
# Create with counter template (default)
typhoon init my-program

# Create with hello-world template
typhoon init simple-program --template hello-world
```

### Creating a Workspace

Create a workspace for projects with multiple programs:

```bash
typhoon init <name> --workspace
```

Generated structure:

```
my-workspace/
├── Cargo.toml
├── programs/
│   └── my-workspace-program/
├── tests/
└── .gitignore
```

### Adding Programs to a Workspace

Add additional programs to an existing workspace:

```bash
typhoon add program <name> [--template <template>]
```

Example:

```bash
cd my-workspace
typhoon add program second-program
```

### Building Programs

```bash
typhoon build
```

Compiles the program to Solana BPF using `cargo build-sbf`. Output binary: `target/deploy/{program_name}.so`.

### Running Tests

```bash
typhoon test
```

### Cleaning Build Artifacts

```bash
typhoon clean
```

## Project Structure

### Single Program

```
my-program/
├── Cargo.toml
├── src/
│   └── lib.rs
├── build.rs
├── tests/
│   └── integration.rs
└── .gitignore
```

Note: `build.rs` is included only in the counter template.

After building, program artifacts appear in `target/deploy/`:
- `my_program-keypair.json`
- `my_program.so`

### Workspace

```
my-workspace/
├── Cargo.toml
├── programs/
│   ├── program-one/
│   └── program-two/
├── tests/
└── .gitignore
```

Program artifacts for each program appear in `target/deploy/` after building.

## Templates

### Counter (Default)

A full-featured template demonstrating account state management, multiple instructions (initialize, increment, close), context-based validation, and IDL generation. Suitable for production programs requiring on-chain state.

### Hello-World

A minimal template with a single instruction and no account state. Ideal for quick prototyping and learning the basics.

## Keypairs

Program keypairs are automatically generated at `target/deploy/{program_name}-keypair.json`. Note that hyphens in program names are converted to underscores in keypair filenames to match Solana's binary naming convention.

The derived program ID is embedded in generated code:

```rust
program_id!("YourGeneratedProgramIdHere");
```

## Dependency Management

The CLI automatically detects whether you're inside the Typhoon repository. Inside the repository, it uses path dependencies for local development. Outside the repository, it uses the published version from crates.io.

```toml
# Inside Typhoon repository
typhoon = { path = "../../crates/lib" }

# Outside Typhoon repository
typhoon = "0.1.0-alpha.16"
```

## Requirements

- Rust 1.82 or later
- Solana CLI tools (provides `cargo build-sbf`)

Install Solana tools:

```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

## Troubleshooting

**Build fails with "cargo build-sbf not found"**
Install Solana CLI tools (see Requirements).

**Keypair not found during build**
Run commands from the project root directory where `target/deploy/` is located.

**Workspace commands fail**
Workspace commands must be run from the workspace root (where the workspace `Cargo.toml` is located).

## License

MIT
