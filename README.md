# hotwired-mcp

MCP (Model Context Protocol) server for [Hotwired](https://hotwired.sh) multi-agent workflow orchestration.

## Why Open Source?

This MCP server runs on your machine and connects to external services. We believe you should be able to:

- **Audit** exactly what code runs on your machine
- **Verify** what data is sent to Hotwired
- **Trust** that there's no hidden behavior
- **Build from source** if you prefer

## Installation

### Via npx (Recommended)

```bash
npx hotwired-mcp
```

### Via npm

```bash
npm install -g hotwired-mcp
hotwired-mcp
```

### Via Cargo (Build from Source)

```bash
cargo install --git https://github.com/hotwired-sh/hotwired-mcp
```

## Usage with Claude Code

Add to your MCP configuration:

```json
{
  "mcpServers": {
    "hotwired": {
      "command": "npx",
      "args": ["hotwired-mcp"],
      "env": {
        "ZELLIJ_SESSION_NAME": "${ZELLIJ_SESSION_NAME}"
      }
    }
  }
}
```

Or use the [Hotwired Claude Plugin](https://github.com/hotwired-sh/claude-plugin) which configures this automatically.

## Prerequisites

- [Hotwired Desktop App](https://hotwired.sh) - Must be running
- [Zellij](https://zellij.dev) - Terminal multiplexer for session management

## How It Works

```
┌─────────────────────────────────────────────────────────────────┐
│  HOTWIRED DESKTOP APP                                           │
│  └── Listens on ~/.hotwired/hotwired.sock                       │
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │ Unix Socket (IPC)
                              │
┌─────────────────────────────┼───────────────────────────────────┐
│  hotwired-mcp               │                                   │
│  ├── Registers session      │                                   │
│  ├── Provides MCP tools     │                                   │
│  └── Relays messages        │                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Available Tools

| Tool | Description |
|------|-------------|
| `get_protocol` | Fetch workflow protocol and role instructions |
| `get_run_status` | Check current run status |
| `report_status` | Update your working state |
| `send_message` | Send message to other participants |
| `request_input` | Ask human for input |
| `report_impediment` | Signal you're blocked |
| `handoff` | Hand work to another agent |
| `task_complete` | Mark a task as complete |

## Development

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test
```

### Run locally

```bash
cargo run
```

## Security

This MCP server:
- Connects only to the local Hotwired socket (`~/.hotwired/hotwired.sock`)
- Does not make external network requests (except to the Hotwired backend via the socket)
- Does not read or modify files outside its scope
- Source code is fully auditable

## License

MIT - See [LICENSE](LICENSE)

## Links

- [Hotwired](https://hotwired.sh) - Multi-agent workflow orchestration
- [MCP Specification](https://modelcontextprotocol.io) - Model Context Protocol
- [Report Issues](https://github.com/hotwired-sh/hotwired-mcp/issues)
