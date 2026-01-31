# Phase 1: Project Setup - Research

**Researched:** 2026-01-31
**Domain:** Tauri 2.0 monorepo project structure, development tooling, and environment setup
**Confidence:** HIGH

## Summary

Phase 1 requires establishing a clean foundation for a Tauri 2.0 application with a three-layer architecture: React/TypeScript frontend, Node.js backend (Hono), and Rust backend (Tauri). The project will use pnpm workspaces for monorepo management and nix-shell for reproducible development environments.

The standard approach is to use `create-tauri-app` as a starting point, then restructure into a monorepo with separate packages for frontend, backend, and shared code. Tauri 2.0 (released stable in 2024) has a mature ecosystem with official templates and well-documented patterns. The key architectural decision is using pnpm workspaces to manage the multi-layer structure while avoiding known Tauri CLI package manager detection issues.

**Primary recommendation:** Initialize with `pnpm create tauri-app` using the React-TypeScript template, then immediately restructure into a monorepo layout with `apps/` and `packages/` directories. Create the Node.js backend as a separate workspace package with Hono, and configure nix-shell with both Node.js and Rust toolchains.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri | 2.x (stable) | Desktop app framework | Official cross-platform framework, active development, mature plugin ecosystem |
| React | 19.x | Frontend UI library | Official Tauri template, excellent TypeScript support, large ecosystem |
| Vite | 6.x | Frontend build tool | Officially recommended by Tauri for React/Vue/Svelte, fast HMR |
| TypeScript | 5.x | Type safety | Industry standard, required for type-safe Tauri commands via tauri-specta |
| pnpm | 9.x | Package manager | Best for monorepos (60-80% less disk usage, 3-5x faster installs), workspace protocol support |
| Hono | 4.x | Node.js server framework | Web standards-based, excellent TypeScript support, lightweight, Node 18+ compatible |
| Nix | 2.x (flakes) | Dev environment | Reproducible environments, multi-language support (Node.js + Rust) |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Tailwind CSS | 4.x | Styling framework | New v4 with Vite plugin - simplified setup, no PostCSS config needed |
| @tailwindcss/vite | latest | Vite integration | Required for Tailwind v4, replaces PostCSS approach |
| ESLint | 9.x | Linting | Use flat config format (eslint.config.js), essential for code quality |
| Prettier | 3.x | Code formatting | Use with eslint-config-prettier to avoid conflicts |
| @tauri-apps/cli | 2.x | Tauri CLI | Development and build commands (tauri dev, tauri build) |
| @tauri-apps/api | 2.x | Frontend API | Frontend bindings for Tauri commands, must match CLI version |
| tauri-specta | 2.x | Type generation | Generate TypeScript bindings from Rust commands for type safety |
| rusqlite | latest | SQLite for Rust | Direct SQLite access in Tauri backend (alternative: tauri-plugin-sql) |
| @hono/node-server | latest | Hono Node.js adapter | Required for running Hono on Node.js runtime |
| Drizzle ORM | latest | Database toolkit | Type-safe SQL, used by universal-agent-harness |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| pnpm workspaces | npm workspaces, yarn workspaces | pnpm has better performance and disk usage, but requires workaround for Tauri CLI bug |
| Hono | Express, Fastify | Hono is more modern with Web Standards API, better TypeScript support |
| rusqlite | Diesel, tauri-plugin-sql | rusqlite is lighter; Diesel adds ORM overhead; plugin adds Tauri dependency |
| Tailwind v4 | Tailwind v3, vanilla CSS | v4 has simpler setup but is newer; v3 more mature if issues arise |
| nix-shell | Docker, mise, asdf | Nix provides true reproducibility but steeper learning curve |

**Installation (after monorepo structure created):**
```bash
# At workspace root
pnpm install

# Tauri CLI (dev dependency in desktop app)
pnpm add -D @tauri-apps/cli -F desktop

# Hono for backend
pnpm add hono @hono/node-server -F backend

# Tailwind v4 for frontend
pnpm add -D tailwindcss @tailwindcss/vite -F desktop
```

## Architecture Patterns

### Recommended Project Structure

```
aios/
├── .planning/                   # GSD planning docs
├── apps/
│   ├── desktop/                 # Tauri app
│   │   ├── src/                 # React/TypeScript frontend
│   │   │   ├── main.tsx
│   │   │   ├── App.tsx
│   │   │   └── index.css
│   │   ├── src-tauri/           # Rust backend
│   │   │   ├── src/
│   │   │   │   ├── lib.rs       # Main entry (NOT main.rs)
│   │   │   │   └── commands/    # Tauri commands
│   │   │   ├── Cargo.toml
│   │   │   ├── tauri.conf.json
│   │   │   └── capabilities/    # Security permissions
│   │   ├── package.json
│   │   ├── vite.config.ts
│   │   ├── tsconfig.json
│   │   └── index.html
│   └── backend/                 # Node.js server (Hono)
│       ├── src/
│       │   ├── index.ts         # Server entry
│       │   ├── routes/          # Route modules
│       │   └── lib/             # Shared logic
│       ├── package.json
│       └── tsconfig.json
├── packages/
│   ├── shared/                  # Shared TypeScript utilities
│   │   ├── src/
│   │   ├── package.json
│   │   └── tsconfig.json
│   └── typescript-config/       # Shared tsconfig bases
│       ├── base.json
│       ├── react.json
│       └── node.json
├── pnpm-workspace.yaml
├── package.json                 # Workspace root
├── tsconfig.json                # Project references root
├── shell.nix or flake.nix       # Nix development environment
└── .gitignore
```

### Pattern 1: pnpm Workspace Configuration

**What:** Monorepo package management with workspace protocol for local dependencies
**When to use:** Always, for managing multiple packages in the repository

**Example:**
```yaml
# pnpm-workspace.yaml
packages:
  - 'apps/*'
  - 'packages/*'
```

```json
// apps/desktop/package.json
{
  "name": "@aios/desktop",
  "dependencies": {
    "@aios/shared": "workspace:*"
  }
}

// Root package.json
{
  "name": "@aios/monorepo",
  "private": true,
  "scripts": {
    "dev": "pnpm --filter desktop dev",
    "build": "pnpm --filter desktop build",
    "dev:backend": "pnpm --filter backend dev"
  }
}
```

### Pattern 2: TypeScript Project References

**What:** Incremental compilation with type dependencies across packages
**When to use:** For faster builds in monorepos with 3+ packages

**Example:**
```json
// tsconfig.json (root - project references entry point)
{
  "files": [],
  "references": [
    { "path": "./apps/desktop" },
    { "path": "./apps/backend" },
    { "path": "./packages/shared" }
  ]
}

// packages/shared/tsconfig.json
{
  "extends": "@aios/typescript-config/base.json",
  "compilerOptions": {
    "composite": true,
    "declaration": true,
    "declarationMap": true,
    "outDir": "./dist"
  }
}

// apps/desktop/tsconfig.json
{
  "extends": "@aios/typescript-config/react.json",
  "references": [
    { "path": "../../packages/shared" }
  ]
}
```

### Pattern 3: Nix Development Shell

**What:** Reproducible development environment with multiple language toolchains
**When to use:** Always, to ensure consistent builds across machines

**Example (flake.nix):**
```nix
{
  description = "AIOS development environment";

  inputs = {
    nixpkgs.url = "github:nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "aarch64-darwin"; # or "x86_64-linux"
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          # Node.js ecosystem
          pkgs.nodejs_22
          pkgs.nodePackages.pnpm

          # Rust ecosystem
          pkgs.cargo
          pkgs.rustc
          pkgs.rust-analyzer
          pkgs.rustfmt

          # Tauri system dependencies (macOS)
          # Linux would need webkit2gtk, etc.

          # Utilities
          pkgs.git
        ];

        shellHook = ''
          echo "AIOS development environment loaded"
          echo "Node: $(node --version)"
          echo "pnpm: $(pnpm --version)"
          echo "Rust: $(rustc --version)"
        '';
      };
    };
}
```

### Pattern 4: Hono Server Structure

**What:** Modular route organization with type-safe RPC
**When to use:** For the Node.js backend to maintain clean separation

**Example:**
```typescript
// apps/backend/src/index.ts
import { serve } from '@hono/node-server'
import { Hono } from 'hono'
import agentsRoute from './routes/agents'

const app = new Hono()

app.route('/agents', agentsRoute)

serve({
  fetch: app.fetch,
  port: 3000,
}, (info) => {
  console.log(`Server running on http://localhost:${info.port}`)
})

export type AppType = typeof app

// apps/backend/src/routes/agents.ts
import { Hono } from 'hono'

const agents = new Hono()
  .get('/', (c) => c.json({ agents: [] }))
  .post('/', (c) => c.json({ created: true }))

export default agents
```

**Source:** https://hono.dev/docs/guides/best-practices

### Pattern 5: Tauri Command Organization

**What:** Type-safe commands using tauri-specta with modular structure
**When to use:** All Tauri backend commands for frontend-backend communication

**Example:**
```rust
// src-tauri/src/lib.rs (NOT main.rs - required for mobile support)
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_agents
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// src-tauri/src/commands/mod.rs
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

**Source:** https://v2.tauri.app/start/project-structure/

### Anti-Patterns to Avoid

- **Using main.rs for commands:** Use lib.rs instead - main.rs won't work for mobile builds
- **Global npm install with nix-shell:** Configure local npm global directory (~/.npm-global) to avoid permission issues
- **Creating Rails-like controllers in Hono:** Write handlers inline to preserve TypeScript type inference for path parameters
- **Mixing package managers:** Stick to pnpm exclusively; mixing with npm corrupts node_modules
- **Forgetting workspace protocol:** Use "workspace:*" for local dependencies, not file paths or versions

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process execution with MCP | Custom child_process wrapper | process-mcp (user library) | Handles TTY, timeouts, Docker/host modes, MCP protocol, already tested |
| Agent orchestration | Custom agent loop | universal-agent-harness (user library) | Provides MCP integration, run tracking, cost tracking, profile/problem management |
| SQLite migrations | Manual .sql files | rusqlite with embedded migrations or Drizzle ORM | Automatic version tracking, rollback support, type safety |
| TypeScript from Rust | Manual type definitions | tauri-specta | Auto-generates TypeScript bindings from Rust types, ensures sync |
| Monorepo task running | Shell scripts | pnpm --filter or Turborepo | Dependency-aware builds, caching, parallel execution |
| Development environment | Manual install instructions | nix-shell with flake.nix | True reproducibility, version pinning, multi-language support |
| Code formatting | Manual style enforcement | Prettier + ESLint flat config | Industry standard, plugin ecosystem, IDE integration |

**Key insight:** The Tauri ecosystem provides most infrastructure needs through official plugins and community libraries. Focus on application logic, not reinventing tooling.

## Common Pitfalls

### Pitfall 1: Tauri CLI Package Manager Detection in Monorepos

**What goes wrong:** Running `tauri add <plugin>` in a pnpm workspace defaults to npm instead of pnpm, creating package-lock.json and corrupting node_modules.

**Why it happens:** Tauri CLI searches for lock files in the current directory, but pnpm-lock.yaml lives at workspace root in monorepos.

**How to avoid:**
- Create an empty `pnpm-lock.yaml` file in `apps/desktop/` to signal pnpm usage
- OR always use `pnpm tauri add <plugin>` and clean up: `rm -rf node_modules package-lock.json && pnpm install`

**Warning signs:**
- Seeing "WARNING: no lock files found, defaulting to npm"
- package-lock.json appearing in workspace packages

**Source:** https://github.com/tauri-apps/tauri/issues/11859

### Pitfall 2: Using main.rs Instead of lib.rs

**What goes wrong:** Tauri commands defined in main.rs work for desktop but fail when building for mobile platforms (iOS/Android).

**Why it happens:** Mobile builds compile the Rust code as a library (libapp.a), which requires lib.rs as the entry point. main.rs is only for desktop executables.

**How to avoid:**
- Always put application logic in `src-tauri/src/lib.rs`
- Use the `#[cfg_attr(mobile, tauri::mobile_entry_point)]` attribute on the run() function
- Keep main.rs minimal (just calls lib.rs)

**Warning signs:**
- Desktop builds work but mobile builds fail
- Error messages about missing mobile entry point

**Source:** https://v2.tauri.app/start/project-structure/

### Pitfall 3: Tauri 2.0 Event System Changes

**What goes wrong:** Events don't trigger listeners when using AnyLabel targets (the default when passing a string to listen/emitTo).

**Why it happens:** Tauri 2.0 redesigned the event system to use explicit targets instead of event source detection. String targets default to AnyLabel which has specific matching behavior.

**How to avoid:**
- Understand the new target-based event system
- Remove event listeners when components unmount to prevent memory leaks
- Be cautious with high-frequency event emission (can crash the app)

**Warning signs:**
- Listeners set up but never triggered
- Events working inconsistently
- App crashes with frequent emit calls

**Source:** https://github.com/tauri-apps/tauri/issues/11561

### Pitfall 4: Tauri 2.0 API Module Reorganization

**What goes wrong:** Import statements from v1 break in v2 (e.g., `@tauri-apps/api/fs` no longer exists).

**Why it happens:** Tauri 2.0 moved most core functionality to separate plugins to allow independent iteration and lower contribution barriers.

**How to avoid:**
- Use `@tauri-apps/api/core` instead of `@tauri-apps/api/tauri`
- Install specific plugins for features: @tauri-apps/plugin-fs, @tauri-apps/plugin-shell, etc.
- Reference the migration guide for import mapping

**Warning signs:**
- Module not found errors for @tauri-apps/api/* imports
- Features that worked in examples don't exist in your project

**Source:** https://v2.tauri.app/start/migrate/from-tauri-1/

### Pitfall 5: TypeScript Project References Path Resolution

**What goes wrong:** Import paths in TypeScript projects don't resolve correctly after extending a base tsconfig.json.

**Why it happens:** Paths in tsconfig.json are relative to the file containing them, not the file doing the extending.

**How to avoid:**
- Define paths in each package's tsconfig.json, not just the base config
- Use workspace protocol in package.json for actual dependency resolution
- Rely on pnpm's symlinking for imports, not TypeScript path mapping

**Warning signs:**
- IDE can't find types from shared packages
- Build succeeds but IDE shows errors
- Imports work in one package but not another

**Source:** https://nx.dev/blog/managing-ts-packages-in-monorepos

### Pitfall 6: Nix Shell Global npm Conflicts

**What goes wrong:** Running `npm install -g` inside nix-shell fails with permission errors or installs to unexpected locations.

**Why it happens:** nix-shell provides Node.js in a read-only store path, and npm's default global directory points to system locations.

**How to avoid:**
- Configure local npm global directory: `mkdir ~/.npm-global && npm config set prefix ~/.npm-global`
- Add ~/.npm-global/bin to PATH in shell.nix shellHook
- Prefer project-local installations with pnpm

**Warning signs:**
- Permission denied errors when installing global packages
- Global packages installed but not in PATH

**Source:** https://gist.github.com/danielres/2b4a0c93fc850832b8c5dd1ad8882a94

### Pitfall 7: ESLint v9 Config Format Change

**What goes wrong:** Using .eslintrc.json format with ESLint 9+ shows deprecation warnings or fails to load config.

**Why it happens:** ESLint 9 promotes flat config (eslint.config.js) as the primary format, deprecating .eslintrc formats.

**How to avoid:**
- Use eslint.config.js (flat config) from the start
- If using Vite template, it may provide this by default
- Ensure eslint-config-prettier is last in extends array

**Warning signs:**
- Deprecation warnings about config format
- ESLint rules not applying correctly

**Source:** https://dev.to/denivladislav/set-up-a-new-react-project-vite-typescript-eslint-prettier-and-pre-commit-hooks-3abn

## Code Examples

Verified patterns from official sources:

### Tauri Command with Type Safety

```rust
// Source: https://v2.tauri.app/develop/calling-rust/
use tauri::State;

#[derive(Default)]
struct AppState {
    counter: std::sync::Mutex<i32>,
}

#[tauri::command]
fn increment_counter(state: State<AppState>) -> i32 {
    let mut counter = state.counter.lock().unwrap();
    *counter += 1;
    *counter
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![increment_counter])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend Calling Rust Command

```typescript
// Source: https://v2.tauri.app/develop/calling-rust/
import { invoke } from '@tauri-apps/api/core'

async function increment() {
  const newValue = await invoke<number>('increment_counter')
  console.log('Counter:', newValue)
}
```

### Hono Route Organization

```typescript
// Source: https://hono.dev/docs/guides/best-practices
import { Hono } from 'hono'
import { zValidator } from '@hono/zod-validator'
import { z } from 'zod'

const schema = z.object({
  name: z.string(),
})

const app = new Hono()
  .get('/', (c) => {
    // Handler directly after path - preserves type inference
    return c.json({ message: 'Hello' })
  })
  .post('/', zValidator('json', schema), (c) => {
    const { name } = c.req.valid('json')
    return c.json({ greeting: `Hello ${name}` })
  })

export default app
```

### pnpm Workspace with TypeScript

```json
// Source: https://pnpm.io/workspaces
// package.json in workspace package
{
  "name": "@aios/desktop",
  "dependencies": {
    "@aios/shared": "workspace:*",
    "react": "^19.0.0"
  }
}
```

Before publishing, `workspace:*` automatically converts to the actual version (e.g., `^1.0.0`).

### Tailwind CSS v4 with Vite

```typescript
// Source: https://tailwindcss.com/docs (v4)
// vite.config.ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()]
})
```

```css
/* src/index.css */
@import "tailwindcss";
```

### Universal Agent Harness Integration

```typescript
// Source: ~/dev/universal-agent-harness/README.md
import { createRun, run } from 'universal-agent-harness';

// Create a run with profile and problem
const result = await createRun({
  name: "background-task",
  problemId: "user-task",
  model: "claude-sonnet-4-5",
  agentCount: 1,
  profile: "aios-agent"
});

// Run with callback for UI updates
await run({
  runName: "background-task",
  onMessage: (msg) => {
    // Send to frontend via Tauri events
    console.log("Agent:", msg)
  },
  onCostUpdate: (cost) => {
    console.log("Cost:", cost)
  }
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 API organization | Tauri v2 plugin architecture | Oct 2024 | Core features now separate plugins, requires migration of imports |
| .eslintrc.json | eslint.config.js (flat config) | ESLint v9 (2024) | Simpler config, better TypeScript support, breaking change |
| Tailwind v3 PostCSS | Tailwind v4 Vite plugin | Nov 2024 | No PostCSS config needed, simpler setup, smaller API surface |
| TypeScript Project References optional | Standard for monorepos 10+ packages | Ongoing | Better incremental builds, but adds configuration complexity |
| npm/yarn dominance | pnpm as monorepo standard | 2023-2024 | Massive disk/speed improvements, workspace protocol is best practice |
| Tauri @tauri-apps/api/tauri | @tauri-apps/api/core | Tauri v2 | Module renamed, breaking import change |

**Deprecated/outdated:**
- **Tauri v1 event system**: Replaced with target-based system in v2
- **Tauri CLI < 2.0**: Plugin installation workflow changed
- **.eslintrc formats**: Use flat config (eslint.config.js) with ESLint 9+
- **Manual Rust→TypeScript types**: Use tauri-specta for auto-generation
- **Vite template with Tailwind v3**: Use v4 for new projects (simpler)

## Open Questions

Things that couldn't be fully resolved:

1. **Optimal TypeScript Project References Configuration**
   - What we know: Turborepo docs suggest avoiding them due to config/caching overhead
   - What's unclear: Whether the build speed benefits outweigh complexity for this 3-layer project
   - Recommendation: Start without project references, add only if build times become problematic

2. **Nix Shell vs Nix Flakes for Dev Environment**
   - What we know: Both work, flakes are "experimental" but widely adopted, provide better reproducibility
   - What's unclear: Whether stable Nix features are sufficient for this use case
   - Recommendation: Use flakes (they're de facto standard in 2026 despite experimental status)

3. **Backend Integration Point with Tauri**
   - What we know: Tauri frontend can call Node.js backend via HTTP, or both can run as separate processes
   - What's unclear: Best practice for lifecycle management (should Tauri spawn the backend process?)
   - Recommendation: Research during implementation phase - likely spawn from Tauri for unified lifecycle

4. **SQLite Database Location and Sharing**
   - What we know: Tauri provides app_data_dir() for persistent storage; universal-agent-harness uses SQLite
   - What's unclear: Should agent harness database be in Tauri app data, or separate? How to configure paths?
   - Recommendation: Use Tauri's path API for all persistent data, configure agent harness to use Tauri paths

## Sources

### Primary (HIGH confidence)

- [Tauri v2 Create Project Guide](https://v2.tauri.app/start/create-project/) - Official setup instructions
- [Tauri v2 Project Structure](https://v2.tauri.app/start/project-structure/) - Directory layout and lib.rs requirement
- [Tauri v2 Prerequisites](https://v2.tauri.app/start/prerequisites/) - System dependencies
- [Tauri v2 CLI Reference](https://v2.tauri.app/reference/cli/) - Command documentation
- [pnpm Workspaces Documentation](https://pnpm.io/workspaces) - Workspace protocol and configuration
- [Hono Node.js Getting Started](https://hono.dev/docs/getting-started/nodejs) - Official Node.js setup
- [Hono Best Practices](https://hono.dev/docs/guides/best-practices) - Route organization patterns
- [Tailwind CSS v4 Installation](https://tailwindcss.com/docs) - Vite plugin setup
- local: ~/dev/universal-agent-harness/README.md - User's agent orchestration library
- local: ~/dev/process-mcp/README.md - User's process execution MCP server

### Secondary (MEDIUM confidence)

- [Tauri + pnpm Workspace Issue #11859](https://github.com/tauri-apps/tauri/issues/11859) - Package manager detection bug and workaround
- [Nx: Managing TypeScript Packages in Monorepos](https://nx.dev/blog/managing-ts-packages-in-monorepos) - Project references best practices
- [the-nix-way/dev-templates](https://github.com/the-nix-way/dev-templates) - Nix flake templates for multi-language environments
- [Tauri v2 Migration Guide](https://v2.tauri.app/start/migrate/from-tauri-1/) - API changes and plugin architecture
- [WebSearch: Monorepo best practices 2026](https://medium.com/@sanjaytomar717/the-ultimate-guide-to-building-a-monorepo-in-2025-sharing-code-like-the-pros-ee4d6d56abaa) - Structure patterns
- [WebSearch: ESLint flat config 2026](https://dev.to/denivladislav/set-up-a-new-react-project-vite-typescript-eslint-prettier-and-pre-commit-hooks-3abn) - Modern ESLint setup

### Tertiary (LOW confidence)

- Community Tauri templates on GitHub (dannysmith/tauri-template, MrLightful/create-tauri-react) - Useful patterns but not official
- WebSearch results on Tauri SQLite integration - Multiple approaches exist, needs verification during implementation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All recommendations from official documentation or stable packages
- Architecture: HIGH - Patterns verified from official Tauri, pnpm, Hono docs and working examples
- Pitfalls: HIGH - All sourced from official issue trackers, migration guides, or documentation warnings
- Nix integration: MEDIUM - While patterns are standard, optimal configuration for this specific stack may need iteration
- Backend integration: MEDIUM - Architecture is clear but lifecycle management details need implementation testing

**Research date:** 2026-01-31
**Valid until:** 2026-03-31 (60 days - stable stack with mature tools)

**Notes:**
- Tauri 2.0 reached stable in October 2024, ecosystem is mature
- pnpm workspace patterns are well-established and stable
- Hono is actively developed but API is stable for core features
- Tailwind v4 is newest component but has official docs and Vite plugin
- Main unknowns are integration patterns (backend lifecycle, shared database), not tooling choices
