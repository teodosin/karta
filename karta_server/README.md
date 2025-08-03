# Karta Server

The Rust backend for Karta, handling graph database operations and file system integration.

## What it does

This server provides a REST API for managing the graph database that powers Karta. It handles:

- **Graph operations** - Creating, updating, and querying nodes and edges
- **File system integration** - Syncing filesystem changes with graph data
- **Asset management** - Handling image and file assets
- **Context management** - Managing different views/contexts of the same data

The server uses [AGDB](https://github.com/agnesoft/agdb) as the graph database backend, which provides efficient graph storage and querying.

## Architecture

**Core components:**
- `src/server/` - REST API endpoints and routing
- `src/graph_agdb/` - Database layer and AGDB integration
- `src/fs_reader/` - File system integration and watching
- `src/context/` - Context management (RON file handling)
- `src/elements/` - Core data structures and graph elements

**Key concepts:**
- **DataNodes** - Represent files/folders in the filesystem or virtual nodes
- **Edges** - Connections between nodes with typed relationships
- **Contexts** - Different spatial arrangements of the same nodes
- **ViewNodes** - Position and visual state of nodes within contexts

## API Overview

**Core endpoints:**
- `GET /` - Vault information
- `GET /api/asset/{path}` - File assets
- `GET /api/paths` - File system paths
- `GET /api/search` - Search nodes
- `GET /api/contexts` - Available contexts

**Node operations:**
- `POST /api/nodes` - Create new node
- `DELETE /api/nodes` - Delete nodes
- `PUT /api/nodes/{id}` - Update node
- `GET /api/nodes/{id}` - Get node by ID
- `GET /api/nodes/by-path/{path}` - Get node by file path

**Edge operations:**
- `POST /api/edges` - Create edges
- `DELETE /api/edges` - Delete edges
- `PATCH /api/edges` - Reconnect edge

**Context operations:**
- `PUT /api/ctx/{id}` - Save context
- `GET /ctx/{id}` - Open context from filesystem path

## Development

**Prerequisites:**
- Rust (latest stable)

**Running:**
```bash
cargo run
```

The server starts on `http://localhost:7370` by default. It will prompt you to type in a path for your vault. You can use tab to autocomplete, double click to show all available folders. Once you've created some vaults, you can quickly initialise them without typing in the whole path each time. 

**Configuration:**
Currently uses hardcoded paths and settings. Configuration system is planned but not implemented yet.

## Current limitations

- No authentication/authorization
- Hardcoded database location
- Limited error handling in some areas
- File watching could be more robust
- No configuration system yet

This is development-focused right now - it assumes a single local user and doesn't handle multi-user scenarios or security concerns.

## Integration

The server is designed to work with the Karta Svelte client, but could potentially be used by other applications that need graph-based file organization.

Eventually this will be automatically initialised as a separate process by the Tauri app instead of having to be run individually. 
