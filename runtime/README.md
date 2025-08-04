# @karta/runtime

MIT-licensed, embeddable, read-only runtime for rendering Karta bundles in websites and applications.

Scope
- Read-only data flow and rendering primitives
- No editor CRUD or persistence logic (lives in GPL editor packages)
- Clean separation from server/editor code to avoid license contamination

Status
- Package scaffolding in place
- Minimal runtime entry class and barrels created
- Adapters/stores/components are placeholders pending implementation
- Schema validation and guardrails to be wired

Usage (early scaffold)
```ts
import { KartaRuntime } from "@karta/runtime";

const runtime = new KartaRuntime({});
// upcoming:
// await runtime.loadBundle(source);
// runtime.setContext("root");
// runtime.destroy();
```

License
MIT. Editor modules are GPL and compose runtime components without subclassing.