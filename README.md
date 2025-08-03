# Karta

A node-based file organization tool for creative projects.

> [!IMPORTANT] 
> This project is in active development and not ready for production use. It's functional but still rough around the edges. The current focus is getting the core features stable before expanding functionality.

## What is this?

Karta turns your file system into a spatial graph. Instead of just folders and files, you get nodes and connections. You can arrange your project files on a canvas, create arbitrary links between them, and view your work from different "contexts" - basically different perspectives that show the same files arranged differently.

It's designed for complex creative projects where you need to see relationships between many different files and ideas. Think game development, world-building, writing projects with lots of interconnected parts.

## Current state

**What works:**
- Basic node-based file visualization
- Creating connections between files
- Context switching (viewing the same network from different focal points)
- Server integration with graph database
- File system integration

**What doesn't work yet:**
- Packaged desktop app (currently requires running server and client separately)
- Export/sharing functionality
- Custom node types
- Smooth performance with large vaults and images

## Licensing (work in progress)

The licensing situation is being sorted out to enable both open source development and commercial use of created content. Currently everything is private while we figure this out, but the plan is:

- **Editor**: GPL - keeps the editing tools open source
- **Runtime**: MIT - allows commercial use of exported networks

Both will be part of this repository but serve different purposes. If you're interested in contributing or using this, please reach out so we can discuss the licensing before you invest time. 

## Architecture

This is a monorepo with two main parts:

**`karta_server/`** - Rust backend
- Uses [agdb](https://github.com/agnesoft/agdb) graph database for storage
- Handles the core graph operations and file system integration
- See [karta_server/README.md](./karta_server/README.md)

**`karta_svelte/`** - Desktop application (Tauri + SvelteKit)
- SvelteKit frontend with Tauri wrapper for desktop functionality
- Currently works with the separate server, but the plan is to bundle the server into the Tauri app
- For now you need to run both server and client separately during development
- See [karta_svelte/README.md](./karta_svelte/README.md)

The long-term plan includes splitting functionality into editor and runtime components to enable embedding Karta networks in other applications, while keeping both in this repository.

## Development setup

Prerequisites: Rust, Node.js 18+, and your package manager of choice.

```bash
# Server (in one terminal)
cd karta_server
cargo run

# Client (in another terminal)  
cd karta_svelte
npm install
npm run dev
```

See the individual READMEs for more detailed setup instructions.

## Contributing

It's too early to ask for contributions, but feedback and discussion are very welcome. The project is still finding its direction and having conversations about what this should become is valuable.

If you want to try it out, expect rough edges and incomplete features. But if something clicks for you or you see potential, I'd love to hear about it.

## Background

The motivation is pretty simple: creative projects are interconnected webs of ideas and files, but our tools treat them as isolated pieces in folder hierarchies. This makes it hard to see the relationships and patterns that actually matter for the work.

Karta tries to make those connections visible and interactive. Whether that's actually useful remains to be seen, but it's worth exploring.

---

*Built for creative work that doesn't fit neatly into folders.*


