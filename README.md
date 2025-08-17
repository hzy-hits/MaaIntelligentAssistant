# MAA Intelligent Control Middleware

HTTP server providing AI control interface for MaaAssistantArknights through Function Calling protocol.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Features

- **Function Calling Protocol** - Standard interface for AI providers
- **MAA FFI Integration** - Direct bindings to MaaAssistantArknights
- **Natural Language Interface** - Command translation to game operations
- **Web Interface** - React-based chat interface
- **Docker Support** - Multi-stage containerized deployment

## Quick Start

### One-Click Deployment

```bash
git clone --recursive https://github.com/your-repo/maa-remote-server.git
cd maa-remote-server
./scripts/start-all.sh
```

Access the web interface at http://localhost:3000

### Development Mode

```bash
./scripts/dev.sh
```

### Manual Setup

```bash
# Backend
cargo run

# Frontend (separate terminal)
cd maa-chat-ui && npm run dev
```

## Architecture

```
maa-remote-server/
├── src/                     # Rust backend
│   ├── function_calling_server.rs  # HTTP server
│   ├── maa_adapter/         # MAA FFI bindings
│   ├── mcp_tools/          # Function implementations
│   ├── operator_manager/   # Character data management
│   └── copilot_matcher/    # Strategy matching
├── maa-chat-ui/           # React frontend
├── scripts/               # Deployment scripts
└── Dockerfile             # Container definition
```

## API Reference

### Endpoints

```http
GET  /health         # System health check
GET  /tools          # Available function definitions
POST /call           # Execute function call
GET  /api/*          # Versioned API endpoints
```

### Function Calls

```bash
# Get system status
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_status",
      "arguments": {"verbose": false}
    }
  }'
```

### Available Functions

| Function | Description | Parameters |
|----------|-------------|------------|
| `maa_status` | System status and device info | `verbose`: boolean |
| `maa_command` | Natural language command execution | `command`: string |
| `maa_operators` | Character data management | `query_type`: enum, `query`: string |
| `maa_copilot` | Strategy matching and execution | `copilot_config`: object |

## Configuration

Create `.env` file:

```bash
QWEN_API_KEY=your_api_key
QWEN_API_BASE=https://dashscope.aliyuncs.com/compatible-mode/v1
HTTP_PROXY=http://127.0.0.1:7897  # Optional proxy
```

## Docker Deployment

```bash
# Build and run
docker-compose up

# Or build manually
docker build -t maa-server .
docker run -p 8080:8080 maa-server
```

## System Requirements

- **Platform**: macOS (primary), Linux, Windows
- **Rust**: 1.70+
- **Node.js**: 18+ (for frontend development)
- **Docker**: Optional for containerized deployment

## Technology Stack

### Backend
- Rust
- Axum 0.8.4 (HTTP server)
- Tokio (async runtime)
- Sled (embedded database)
- MAA Core FFI

### Frontend
- React 19
- Vite 5
- CSS variables (theme support)

## Development

### Build

```bash
cargo build --release
```

### Test

```bash
# API health check
curl http://localhost:8080/health

# Function definitions
curl http://localhost:8080/tools

# Execute function
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{"function_call":{"name":"maa_status","arguments":{}}}'
```

### Frontend Development

```bash
cd maa-chat-ui
npm install
npm run dev
```

## Game Context

**Target**: Arknights tower defense game automation
**Operations**: Daily missions, resource farming, operator management
**Integration**: Direct FFI bindings to MaaAssistantArknights

## License

MIT License