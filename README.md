# mcp_rs_test

MCP server implementation in Rust.
Project to learn Rust and the MCP protocol at the same time (why not?).

## Requirements
- Rust (1.83 or later)

## Build

For a debug build:
```bash
cargo build
```

For a release build:
```bash
cargo build --release
```

## Installation
To install the server, just add the following lines to your `claude_desktop_config.json` file:

```json
{
  "mcp_rs_test": {
    "command": "<path to your mcp_rs_test.exe>",
    "args": []
  }
}
```
