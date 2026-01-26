# sfx-rs

A high-performance, lightweight tool to create self-extracting archives (SFX) using Rust. It utilizes the **Tar** format combined with **Zstd** compression to ensure high compression ratios and rapid decompression speeds.

## Features

*   **High Performance**: Built with Rust, leveraging Zstd for fast and efficient compression/decompression.
*   **Single Executable**: The generated SFX is a standalone executable (`.exe` on Windows) requiring no external dependencies or installation.
*   **Smart Extraction**:
    *   If a directory is compressed, it extracts as a folder.
    *   If a single file is compressed, it extracts directly.
*   **Post-Extraction Scripts**: Support for executing a custom command automatically after extraction (useful for installers).
*   **Silent Operation**: The generated executable runs silently by default, suitable for automated scripts or installers.
*   **CLI Options**: The generated SFX accepts command-line arguments for verbose output, listing files, or changing the output directory.

## Installation & Building

Since this is a Rust project with a workspace structure (CLI + Stub), you need to build the release version to generate the necessary artifacts.

1.  **Prerequisites**: Ensure you have [Rust and Cargo](https://rustup.rs/) installed.
2.  **Build**:
    ```powershell
    cargo build --release
    ```

The executables will be located in `target/release/`:
*   `sfx-cli.exe`: The tool to create archives.
*   `sfx-stub.exe`: The stub used internally (embedded into the archives).

## Usage

### Creating an SFX Archive

Use the `sfx-cli` tool to pack your files.

```powershell
# Basic usage: Pack a folder into an executable
./target/release/sfx-cli.exe -i ./my-folder -o installer.exe

# Pack a file with maximum compression (level 21)
./target/release/sfx-cli.exe -i ./app.exe -o app-sfx.exe --level 21

# Create an installer that runs a script after extraction
./target/release/sfx-cli.exe -i ./dist -o setup.exe -e "setup.bat"
```

**Options:**

*   `-i, --input <PATH>`: Input file or directory to compress.
*   `-o, --output <PATH>`: Output executable name (default: `output.exe`).
*   `-l, --level <NUM>`: Zstd compression level (1-21, default: 3).
*   `-e, --exec <CMD>`: Command to execute after extraction.

### Running the SFX Archive

The generated executable (e.g., `installer.exe`) can be run directly.

**Default Behavior:**
*   Extracts files to the current directory.
*   Silent operation (no output unless an error occurs or the embedded command produces output).
*   If a directory was packed, it creates that directory.
*   If an `--exec` command was specified during creation, it runs immediately after extraction.

**Command Line Arguments:**
You can pass arguments to the generated executable to control its behavior:

```powershell
# Show help
./installer.exe --help

# Verbose mode (show extraction progress)
./installer.exe -v

# List contents without extracting
./installer.exe --list

# Extract to a specific directory
./installer.exe -o "C:\Path\To\Destination"
```

## Project Structure

*   **crates/sfx-cli**: The command-line tool for creating archives. It bundles the `sfx-stub` binary.
*   **crates/sfx-stub**: The lightweight runtime that is prepended to the archive data. It handles decompression and file writing.

## License

[GPLv3](LICENSE)