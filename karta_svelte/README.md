# Karta Client

The desktop application frontend for Karta, built with SvelteKit and Tauri.

## What it does

This is the user interface for Karta - the part you actually interact with. It provides:

- **Graph visualization** - Interactive canvas for viewing and manipulating nodes
- **Context switching** - Navigate between different views of your data
- **File integration** - Drag and drop files, create nodes from filesystem
- **Node editing** - Create, edit, and connect different types of nodes
- **Server communication** - Syncs with the Rust backend

## Architecture

**Key directories:**
- `src/lib/components/` - Reusable UI components
- `src/lib/karta/` - Core stores and state management
- `src/lib/node_types/` - Different node type implementations
- `src/lib/interaction/` - Mouse/keyboard interaction handling
- `src/lib/util/` - Utilities and adapters
- `src-tauri/` - Tauri desktop app configuration

**State management:**
- Uses Svelte stores for reactive state
- Separate stores for nodes, edges, contexts, viewport, etc.
- Persistence layer that connects to the Rust backend server

## Key concepts

**Nodes:** Represent files, folders, or virtual content
**Edges:** Connections between nodes with different relationship types  
**Contexts:** Different spatial arrangements of the same nodes
**Viewport:** Pan/zoom state and visual transforms

The UI is built around the idea of contextual navigation - you're always viewing the graph from some focal point, and nodes are positioned relative to that context.

## Development

**Prerequisites:**
- Node.js 18+
- Rust (for Tauri)

**Running in development:**
```bash
# Install dependencies
npm install

# Start development server (web mode)
npm run dev

# Start Tauri development (desktop mode)
# Note: Tauri npm scripts not set up yet, use cargo directly:
cd src-tauri
cargo tauri dev
```

**Building:**
```bash
# Build web version
npm run build

# Build Tauri desktop app
cd src-tauri
cargo tauri build
```

## Current state

**What works well:**
- Basic graph visualization and interaction
- Context switching with smooth transitions
- File node creation and editing
- Server integration with graph database

**Known issues:**
- Performance with large numbers of nodes/images
- Some UI elements need polish
- Error handling could be more robust
- Mobile/touch interaction is limited

## Configuration

The app connects to the Rust backend server running on localhost:7370. There's legacy IndexedDB adapter code that will be removed in a future version.

**Current setup:**
- Uses ServerAdapter to connect to the backend
- Requires the karta_server to be running separately
- Configuration options are minimal and mostly hardcoded

## Tauri integration

This is designed as a desktop application using Tauri. The web version works for development, but the intended experience is as a native desktop app that will eventually bundle the server component.

Current Tauri features:
- Native file system access
- Desktop notifications
- Window management

Planned Tauri features:
- Bundled server process
- Better file watching
- System tray integration
