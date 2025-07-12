# QRMax

Prototype Model Context Protocol (MCP) project that allows QR code generation and decoding. QR Codes uploaded to [catbox.moe](https://catbox.moe).

## Minimum Supported Rust Version (MSRV)

This project requires **Rust 1.82.0** or later.

## Installation

- You can download the aarch64-apple-darwin build through the releases page (therefore `cargo binstall` supports this architecture)
- Run `cargo install qrmax` otherwise.
- The binary will be available at `$HOME/.cargo/bin` for macOS/Linux platforms, usually, and `%USERPROFILE%\.cargo\bin` on Windows.

## Compilation

This project is built using Rust.

```sh
cargo build --release # File will be in the /target/release folder
```

## Run Instructions

The MCP Server has only been tested against Claude Desktop. Please modify your `claude_desktop_config.json` file (accessible through the Settings -> Developer options) to include qrmax.

**Example:**
```json
{
  "mcpServers": {
    "qrmax": {
      "command": "/Users/{your_username}/.cargo/bin/qrmax"
    }
  }
}
```