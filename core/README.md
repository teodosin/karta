# @karta/core

MIT-licensed shared foundations for the Karta ecosystem. Provides types, small utilities, viewport math, constants, and base read-only store classes used by both the GPL editor and the MIT runtime.

Installation will be handled at the monorepo root with pnpm. The svelte store types error in editors is expected until deps are installed.

Contents:
- src/constants.ts — shared constants and runtime version
- src/types/ — core data structures (DataNode, Context, Edge, KartaBundle)
- src/utils/ — small math helpers
- src/viewport/ — viewport transform helpers
- src/stores/baseNodeStore.ts — base read-only node store
- src/stores/baseContextStore.ts — base read-only context store

Build:
- pnpm -w i
- pnpm -w run build