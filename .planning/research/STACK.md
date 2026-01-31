# Technology Stack

**Project:** AIOS - Spotlight-style Agent Launcher
**Researched:** 2026-01-31
**Overall Confidence:** HIGH

## Executive Summary

This stack builds on proven foundations from aios-chat-browser while integrating universal-agent-harness and process-mcp. The three-layer architecture (React frontend → Node.js backend → Rust/Tauri) is validated and necessary because AI SDK and MCP clients require Node.js runtime. All recommendations use current 2026 versions with active maintenance.

---

## Core Framework

| Technology | Version | Purpose | Why This Version |
|------------|---------|---------|------------------|
| **Tauri** | 2.x (stable) | Desktop app framework (Rust backend + WebView) | Tauri 2.0 stable released late 2024, plugin system matured, 35% YoY adoption growth. Cross-platform but optimize for macOS. |
| **React** | 19.1.0+ | Frontend UI framework | Current stable. React 19 includes stable server components (not used here) but DO NOT use RSC - they're for web/SSR, incompatible with desktop apps. Use client-only React. |
| **TypeScript** | ~5.8.3 | Type safety across stack | Current stable with improved type inference. Mandatory everywhere - never use `any`. |
| **Vite** | 7.0.4+ | Frontend build tool | Vite 7 released Jan 2026: 45% faster cold starts, 3-16x faster production builds, Rolldown bundler (experimental but promising), Node 20.19+ requirement. |
| **Tailwind CSS** | 4.0.0+ | Styling framework | v4 released 2024: 5x faster full builds, 100x faster incremental (microseconds). Use `@tailwindcss/vite` plugin - no PostCSS needed. Auto-discovers templates, no config required. |

**Rationale for three-layer architecture:**
- **Frontend (React):** UI rendering, user interaction, Tauri IPC calls
- **Node Backend (Hono):** AI SDK streaming, MCP client connections, universal-agent-harness orchestration
- **Tauri Backend (Rust):** SQLite persistence, native OS features (notifications, screenshots, system tray), window management

**Why Node backend is mandatory:**
- Vercel AI SDK requires Node.js runtime
- MCP TypeScript SDK and clients require Node.js
- universal-agent-harness is a TypeScript library requiring Node.js
- Cannot run these in Rust or browser - Node sidecar is the correct pattern

---

## Backend Layer (Node.js Sidecar)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **Hono** | Latest (4.x+) | Web framework for backend API | Ultrafast, minimal footprint (~30KB), TypeScript-first, runs anywhere (Node/Bun/Deno/Edge). Benchmarks show it's faster than Express, good enough for local sidecar. Use for `/api/chat` streaming endpoint. |
| **Vercel AI SDK** | 6.x (ai package) | LLM streaming, tool calling, agent framework | AI SDK 6 released 2026 with v3 Language Model Spec, agent support, SSE streaming (replaces custom protocol), tool approval, automatic tool input streaming. Integrates with universal-agent-harness. |
| **MCP TypeScript SDK** | 1.x (stable) | MCP client/server implementation | Official TypeScript SDK. v2 planned Q1 2026 but v1.x recommended for production until v2 stable. Provides types for messages, tools, resources, STDIO/SSE transports. |

**Agent Orchestration:**
- **universal-agent-harness** (user's library, ~/dev/universal-agent-harness): Tick-based agent runtime, multi-provider LLM support, MCP integration via profiles, SQLite persistence, cost tracking. This is the core agent runtime - DO NOT build custom agent loop.
- **process-mcp** (user's library, ~/dev/process-mcp): MCP server for process execution in dual modes (Docker sandboxed + host mode). TTY support, background processes, stdin/stdout streaming.

**MCP Servers to integrate:**
- **process-mcp** (Docker + host modes) - computer interaction
- **dynamic-ui-mcp** (if exists) or custom MCP for declarative UI rendering
- Optional: perplexity-mcp for web search, filesystem-mcp for file access

**Installation:**
```bash
# In src-tauri/sidecars/node-backend/
pnpm add hono ai @ai-sdk/anthropic @ai-sdk/openai
pnpm add @modelcontextprotocol/sdk
# universal-agent-harness via local path (~/dev/universal-agent-harness)
# process-mcp via local path (~/dev/process-mcp)
```

**Confidence:** HIGH - All components verified as current 2026 standards.

---

## Frontend Layer (React/TypeScript)

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| **@tauri-apps/api** | 2.x | Tauri frontend bindings | Official Tauri 2.0 API. Use for IPC commands, events, window management. |
| **@tauri-apps/plugin-***  | 2.x | Tauri plugins | Official plugins for notification, filesystem, opener. Install as needed. |
| **Zustand** | 5.0.0+ | State management | Lightweight (~3KB), no boilerplate, no providers, async-native. 30% YoY growth, ~40% adoption in React projects. Ideal for desktop apps. DO NOT use Redux (overkill) or Context API (performance issues). |
| **Lucide React** | 0.400.0+ | Icon library | Lightweight, tree-shakeable, actively maintained. Good for desktop UIs. |
| **Recharts** | 3.7.0+ | Charting library | Most reliable React chart library, built on React + D3 SVG, declarative components, lightweight. For agent-generated data visualizations. |
| **React Markdown** | 10.1.0+ | Markdown rendering | For rendering agent outputs. Use with `remark-gfm`, `remark-math`, `rehype-katex`, `rehype-highlight`. |
| **KaTeX** | 0.16.28+ | Math rendering | For LaTeX in markdown. Lighter than MathJax. |

**Dynamic UI Rendering:**
- Use pattern from aios-chat: agents write JSX/TSX source code as strings
- Frontend compiles at runtime using Babel Standalone (babel.min.js)
- Render in sandboxed iframe for security
- Provide component library (Recharts, Lucide, basic layout components)
- Hot-swappable architecture - agent updates component, frontend re-renders

**Alternatives considered but rejected:**
- **@assistant-ui/react:** Great for chat UIs, but AIOS is a launcher, not a chat app. Boards are custom, not thread-based.
- **React Query/TanStack Query:** Not needed. Agent runtime handles server state, Zustand handles client state, no REST APIs to cache.
- **Redux Toolkit:** Overkill for desktop app. Zustand is sufficient.

**Installation:**
```bash
pnpm add @tauri-apps/api @tauri-apps/plugin-notification @tauri-apps/plugin-opener
pnpm add zustand lucide-react recharts
pnpm add react-markdown remark-gfm remark-math rehype-katex rehype-highlight katex
```

**Confidence:** HIGH - All libraries verified as current and actively maintained.

---

## Tauri Backend Layer (Rust)

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| **tauri** | 2.x | Core Tauri framework | Tauri 2.0 stable, mature plugin system, excellent security model (capability-based permissions). |
| **tauri-plugin-notification** | 2.0.0 | OS notifications | Official plugin. Replaces removed @tauri-apps/api/notification module. Send native OS notifications when agents complete tasks or need input. |
| **tauri-plugin-opener** | 2.0.0 | Open files/URLs | Official plugin. For opening files in default apps. |
| **tauri-plugin-fs** | 2.x | Filesystem access | Official plugin. Secure file reading/writing with permission-based security model. Prevents path traversal attacks. |
| **rusqlite** | 0.32+ | SQLite database | Mature, stable, "bundled" feature compiles SQLite statically (no system dependency). For runs, messages, memory, hooks persistence. |
| **sqlite-vec** | 0.1.x | Vector search extension | First stable release v0.1.0 (Aug 2024). Pure C, zero dependencies, runs anywhere SQLite runs. Pre-v1 but production-ready for testing. Supports f32/f16/bf16/int8/uint8/1bit vectors, multiple distance metrics, SIMD acceleration. Integrate via rusqlite FFI. |
| **tokio** | 1.x | Async runtime | Standard async runtime for Rust. Use "full" features for timers, fs, process, etc. |
| **serde** + **serde_json** | 1.x | Serialization | Standard for JSON IPC between Rust and frontend. Use `derive` feature. |
| **anyhow** | 1.x | Error handling in commands | Ergonomic error handling for Tauri commands. |
| **thiserror** | 2.x | Custom error types | Define custom error types with good error messages. |
| **chrono** | 0.4.x | Date/time handling | With "serde" feature for timestamp serialization. |
| **uuid** | 1.x | Unique IDs | Use "v4" feature for random UUIDs (agent runs, messages). |
| **keyring** | 3.x | Secure credential storage | For API keys. Uses OS keychain (macOS Keychain, Windows Credential Manager). Better than localStorage. |
| **image** | 0.25.x | Screenshot processing | For screenshot capture as context for agents. |

**Event-driven system (hooks/triggers):**

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| **tokio-cron-scheduler** | Latest | Cron job scheduling | Tokio-based, supports cron expressions with second precision (7-field), timezone-aware via chrono, runtime job management (add/remove/start/stop). Optional PostgreSQL/Nats persistence (not needed for local app). |
| **notify** | 6.x+ | File system watching | Cross-platform fs notification library. Used by cargo-watch, rust-analyzer, deno, mdBook. Provides `recommended_watcher()` that auto-selects best backend (inotify on Linux, FSEvents on macOS). WARNING: Increase inotify limits on Linux for large file sets. |
| **axum** or **warp** | axum 0.7+ | Webhook HTTP server | For webhook triggers. Run local HTTP server, receive POST requests, trigger agents. Axum is modern, ergonomic, built on tokio. Alternative: warp (also good, slightly different API). |

**OS-specific (macOS):**
- **tauri-plugin-spotlight** (if exists/needed): For spotlight-like keyboard shortcut registration. Check https://github.com/zzzze/tauri-plugin-spotlight
- Global keyboard shortcuts: Use Tauri's global-shortcut plugin or register via macOS APIs

**FTS5 for keyword search:**
- Enabled in rusqlite via `bundled` feature (includes FTS5 by default in modern SQLite)
- Create FTS5 virtual table for agent memory: `CREATE VIRTUAL TABLE memory_fts USING fts5(content, metadata);`

**Vector search setup:**
```rust
// In Cargo.toml
sqlite-vec = "0.1"
rusqlite = { version = "0.32", features = ["bundled"] }

// In code
use sqlite_vec::sqlite3_vec_init;
// Register extension via rusqlite FFI
// See: https://alexgarcia.xyz/sqlite-vec/rust.html
```

**Installation:**
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-notification = "2.0.0"
tauri-plugin-opener = "2.0.0"
tauri-plugin-fs = "2"
rusqlite = { version = "0.32", features = ["bundled"] }
sqlite-vec = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
keyring = "3"
image = "0.25"
tokio-cron-scheduler = "*"  # Check crates.io for latest
notify = "6"
axum = "0.7"  # For webhook triggers
```

**Confidence:** HIGH - All crates verified as current and actively maintained. sqlite-vec is pre-v1 but stable enough for production testing.

---

## Development Environment

| Tool | Version | Purpose | Why |
|------|---------|---------|-----|
| **Nix** | Latest | Dev environment reproducibility | Project uses nix-shell for dependencies. ALWAYS run commands inside nix-shell. |
| **pnpm** | Latest | Package manager | Faster than npm, workspace support for monorepo (root + node-backend). |
| **Node.js** | 20.19+ or 22.12+ | JavaScript runtime | Vite 7 requires Node 20.19+/22.12+. Node 18 EOL April 2025. |
| **Rust** | Latest stable | Systems programming | Via rustup. Tauri 2.0 works with stable Rust. |
| **ESLint** | 9.x | Linting (TypeScript/React) | Flat config format (9.x). Use typescript-eslint, react-hooks plugin. |
| **Vitest** | 2.x | Testing framework | Vite-native test runner. Fast, modern. |
| **Concurrently** | 9.x | Run dev servers in parallel | `pnpm dev:all` runs node backend + Tauri dev in one command. |

**Build commands:**
```bash
# Enter nix shell
nix-shell

# Install dependencies
pnpm install
cd src-tauri/sidecars/node-backend && pnpm install

# Development (two terminals)
pnpm dev:node    # Terminal 1: Node backend on port 3001
pnpm tauri dev   # Terminal 2: Tauri app

# Or use concurrently (may have output issues)
pnpm dev:all

# Production build
pnpm tauri build
```

**Confidence:** HIGH - All tools verified as current.

---

## Database Schema Design

**SQLite tables (in Tauri Rust layer):**

```sql
-- Agent runs (background tasks)
CREATE TABLE runs (
  id TEXT PRIMARY KEY,  -- UUID
  agent_profile TEXT NOT NULL,  -- universal-agent-harness profile name
  task TEXT NOT NULL,  -- User's original task description
  status TEXT NOT NULL,  -- pending, running, completed, failed, needs_input
  created_at INTEGER NOT NULL,  -- Unix timestamp
  updated_at INTEGER NOT NULL,
  completed_at INTEGER
);

-- Messages (agent conversation history)
CREATE TABLE messages (
  id TEXT PRIMARY KEY,  -- UUID
  run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
  role TEXT NOT NULL,  -- user, assistant, system
  content TEXT NOT NULL,  -- JSON serialized content
  timestamp INTEGER NOT NULL,
  FOREIGN KEY (run_id) REFERENCES runs(id)
);

-- Memory (extracted facts for semantic search)
CREATE TABLE memory (
  id TEXT PRIMARY KEY,  -- UUID
  content TEXT NOT NULL,  -- The fact/memory text
  embedding BLOB NOT NULL,  -- Vector embedding (via sqlite-vec)
  metadata TEXT,  -- JSON metadata (source run_id, timestamp, tags)
  created_at INTEGER NOT NULL
);

-- FTS5 for keyword search on memory
CREATE VIRTUAL TABLE memory_fts USING fts5(content, metadata);

-- Vector index for semantic search
-- Use sqlite-vec virtual table (syntax TBD, see sqlite-vec docs)

-- Event hooks/triggers
CREATE TABLE hooks (
  id TEXT PRIMARY KEY,  -- UUID
  type TEXT NOT NULL,  -- cron, webhook, file_watcher, os_event
  config TEXT NOT NULL,  -- JSON config (cron expr, webhook path, file patterns, OS event type)
  agent_profile TEXT NOT NULL,  -- Which agent to trigger
  task_template TEXT NOT NULL,  -- Task to execute when triggered
  enabled INTEGER NOT NULL DEFAULT 1,  -- 0 or 1
  created_at INTEGER NOT NULL
);

-- Hook executions (audit log)
CREATE TABLE hook_executions (
  id TEXT PRIMARY KEY,  -- UUID
  hook_id TEXT NOT NULL REFERENCES hooks(id) ON DELETE CASCADE,
  run_id TEXT REFERENCES runs(id) ON DELETE SET NULL,  -- Resulting agent run
  triggered_at INTEGER NOT NULL,
  status TEXT NOT NULL,  -- success, failed
  FOREIGN KEY (hook_id) REFERENCES hooks(id)
);
```

**Confidence:** HIGH - Schema informed by universal-agent-harness patterns and sqlite-vec requirements.

---

## What NOT to Use (Anti-Recommendations)

| Technology | Why NOT |
|------------|---------|
| **Next.js / Remix / SSR frameworks** | SSR doesn't work in Tauri. Use Vite + React (client-only). Tauri docs explicitly warn against this. |
| **React Server Components** | RSC requires server environment. Desktop apps are client-only. React 19 includes RSC but ignore it for Tauri. |
| **Electron** | Heavier than Tauri (bundles Chromium + Node), slower, larger binaries. Tauri uses OS WebView, Rust backend is faster and smaller. |
| **Redux / Redux Toolkit** | Overkill for desktop app. Zustand is sufficient and lighter. |
| **MobX** | More complex than Zustand, less adoption in 2026. |
| **PostgreSQL / MySQL** | Unnecessary for local-only app. SQLite is lighter, embedded, no server process. Vector search via sqlite-vec. |
| **Prisma** | Adds overhead for simple local SQLite. rusqlite is faster, lower-level, more control. |
| **Express.js** | Slower than Hono, more boilerplate, older patterns. Hono is modern, faster, TypeScript-first. |
| **Fastify** | Good but Hono is lighter and cross-runtime. Fastify is Node-specific. |
| **Custom agent loop** | DO NOT build custom agent orchestration. Use universal-agent-harness - it's already built, tested, and handles edge cases. |
| **WebSockets for agent communication** | Unnecessary complexity. Node backend runs locally as sidecar, HTTP is sufficient. AI SDK uses SSE for streaming. |

**Confidence:** HIGH - Anti-recommendations based on Tauri best practices and ecosystem research.

---

## Spotlight-Style Launcher Pattern

**Keyboard shortcut registration:**
- Use Tauri global-shortcut plugin or tauri-plugin-spotlight
- Register CMD+Space (or CMD+Shift+Space to avoid macOS Spotlight conflict)
- Show window on shortcut, hide on Escape or focus loss

**Window configuration (Tauri):**
```json
{
  "windows": [{
    "label": "launcher",
    "visible": false,
    "decorations": false,
    "alwaysOnTop": true,
    "center": true,
    "width": 600,
    "height": 400,
    "transparent": true,
    "resizable": false
  }]
}
```

**Pattern:**
1. User presses shortcut → Tauri shows launcher window
2. User types task → Frontend sends to Node backend
3. Node backend → universal-agent-harness starts run
4. Frontend closes launcher, shows floating indicator (fades after 5s)
5. Agent runs in background
6. On completion/needs_input → Tauri sends OS notification
7. User clicks notification → Opens board UI for that run

**Reference implementation:**
- https://github.com/ahkohd/tauri-macos-spotlight-example
- https://github.com/zzzze/tauri-plugin-spotlight

**Confidence:** MEDIUM - Pattern is well-established, but tauri-plugin-spotlight may need evaluation for compatibility with Tauri 2.0.

---

## Horizontal Board UI Pattern

**Layout:**
- CSS Grid with horizontal scroll (overflow-x: auto)
- Each board is a full-viewport-width panel
- CMD+K search to jump between boards
- First board: Agent management (all runs)
- Subsequent boards: Individual run outputs

**Implementation:**
```tsx
// Zustand store
interface Board {
  id: string;
  type: 'management' | 'run';
  runId?: string;
  title: string;
}

// CSS
.board-container {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: 100vw;
  overflow-x: auto;
  scroll-snap-type: x mandatory;
  scroll-behavior: smooth;
}

.board {
  scroll-snap-align: start;
  width: 100vw;
  height: 100vh;
  overflow-y: auto;
}
```

**CMD+K search:**
- Use cmdk library (https://github.com/pacocoursey/cmdk) or custom implementation
- Index: All runs, all boards, quick actions
- Jump to board on select

**Confidence:** HIGH - Pattern is common in productivity apps (Linear, Height, Notion).

---

## API Integration Pattern

**Frontend → Node Backend:**
```typescript
// Frontend (React)
const response = await fetch('http://localhost:3001/api/chat', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ task, profile: 'default' })
});

// Node Backend (Hono)
app.post('/api/chat', async (c) => {
  const { task, profile } = await c.req.json();
  // Use universal-agent-harness to start run
  const run = await agentRuntime.startRun(profile, task);
  // Stream results via AI SDK
  return streamText({...});
});
```

**Frontend → Tauri Backend (IPC):**
```typescript
// Frontend
import { invoke } from '@tauri-apps/api/core';

const runs = await invoke('get_all_runs');
await invoke('save_message', { runId, role, content });

// Rust
#[tauri::command]
async fn get_all_runs() -> Result<Vec<Run>> {
  // Query SQLite
}
```

**Tauri Events (notifications):**
```typescript
// Rust → Frontend event
window.emit('run_completed', { runId, status });

// Frontend listener
import { listen } from '@tauri-apps/api/event';
await listen('run_completed', (event) => {
  showNotification(event.payload);
});
```

**Confidence:** HIGH - Patterns are standard Tauri + Hono practices.

---

## Security Considerations

**Agent isolation:**
- process-mcp in Docker mode provides sandboxing
- Host mode requires user confirmation for destructive operations
- MCP servers follow least-privilege: only grant necessary capabilities

**API key storage:**
- DO NOT use localStorage for API keys (frontend can't access OS keychain)
- Use Rust keyring crate → Tauri command → Frontend calls `invoke('get_api_key')`
- Keys stored in macOS Keychain, Windows Credential Manager

**Dynamic UI rendering security:**
- Compile agent-generated JSX in sandboxed iframe
- CSP headers to prevent XSS
- No `dangerouslySetInnerHTML` - use React Markdown with rehype sanitization

**SQLite injection prevention:**
- Use parameterized queries in rusqlite (no raw SQL concatenation)
- Vector embeddings are binary blobs, not user-controlled SQL

**Confidence:** HIGH - Standard security practices for desktop apps.

---

## Performance Considerations

**Startup time:**
- Tauri apps start fast (~100ms on modern machines)
- Node backend sidecar adds ~1-2s startup time (acceptable for desktop app)
- Lazy-load boards (only render visible board)

**Memory usage:**
- Tauri WebView uses ~50-100MB base
- Node backend (Hono + AI SDK) ~100-200MB
- Rust backend ~10-20MB
- Total: ~200-400MB (acceptable for desktop app, far less than Electron)

**Database query optimization:**
- Index run_id, status, created_at for fast queries
- FTS5 for keyword search is very fast (no external search engine needed)
- sqlite-vec for vector similarity (slower than dedicated vector DBs but acceptable for local app with <100K memories)

**Streaming performance:**
- AI SDK SSE streaming is efficient
- Use Zustand's `shallow` equality for selective re-renders
- Virtualize long message lists (react-window or react-virtual)

**Confidence:** HIGH - Tauri 2.0 performance is well-documented and excellent.

---

## Migration from aios-chat-browser

**Carry forward:**
- Tauri 2.0 + React + TypeScript + Vite + Tailwind setup
- Dynamic UI rendering pattern (agent writes TSX → frontend compiles → render)
- Theme system (CSS variables for dark/light mode)
- Zustand state management
- Tauri IPC patterns for native features

**Remove/Replace:**
- @assistant-ui/react → Custom board UI
- Ad-hoc AI SDK integration → universal-agent-harness
- Custom MCP setup → harness's profile-based MCP config
- Chat window → Spotlight launcher

**New additions:**
- universal-agent-harness integration
- process-mcp dual-mode support
- Horizontal board scroller
- Event hooks/triggers system (cron, webhooks, file watchers, OS events)
- sqlite-vec for semantic memory search
- OS notifications via tauri-plugin-notification
- Spotlight-style keyboard shortcuts

**Confidence:** HIGH - Migration path is clear based on PROJECT.md context.

---

## Deployment & Distribution

**Development:**
- `pnpm tauri dev` builds debug binary
- Node backend runs as separate process (http://localhost:3001)

**Production:**
- `pnpm tauri build` creates optimized binary
- Node backend bundled as Tauri sidecar (enable tauri-plugin-shell in Cargo.toml)
- Sidecar executable included in app bundle (.app on macOS)
- Tauri spawns sidecar on app launch, kills on app exit

**macOS distribution:**
- .app bundle (for drag-to-Applications)
- .dmg installer (for distribution)
- Code signing required for macOS Gatekeeper (Apple Developer account)
- Notarization required for macOS 10.15+ (submit to Apple)

**Auto-update:**
- tauri-plugin-updater for app updates
- Check GitHub releases or custom update server
- Download .app.tar.gz, verify signature, replace app

**Confidence:** MEDIUM - Tauri build/distribution is mature but macOS notarization requires Apple Developer account setup.

---

## Open Questions & Research Flags

1. **tauri-plugin-spotlight compatibility:** Check if https://github.com/zzzze/tauri-plugin-spotlight works with Tauri 2.0. May need to fork or implement custom global-shortcut handling.

2. **universal-agent-harness integration specifics:** Need to review harness's API for:
   - Starting/stopping runs
   - Progress streaming
   - MCP profile configuration
   - SQLite schema expectations

3. **process-mcp dual mode configuration:** Confirm how to configure Docker vs host mode in MCP profile. Check if harness supports mode switching per-tool.

4. **dynamic-ui-mcp status:** Confirm if dynamic-ui-mcp exists as separate library or needs to be custom-built. Pattern is clear (agent returns JSX, frontend compiles), but MCP server integration needs validation.

5. **sqlite-vec production readiness:** v0.1.0 is "stable" but pre-v1. Monitor for breaking changes. v1.0 planned within ~1 year. Consider fallback to keyword-only search if vector search has issues.

6. **Webhook server port conflicts:** axum HTTP server for webhooks needs port allocation strategy. Use random available port or user-configurable port? How to expose to external webhooks (ngrok, cloudflare tunnel)?

7. **macOS-specific APIs:** File watcher (notify) works cross-platform, but OS event triggers (e.g., "on system wake", "on network change") may need macOS-specific APIs. Research NSWorkspace notifications or use launchd for some events.

**Confidence on open questions:** MEDIUM-LOW - These need phase-specific research during implementation.

---

## Sources

### Tauri & Desktop
- [Tauri 2.0 Stable Release](https://v2.tauri.app/blog/tauri-20/)
- [Tauri 2.0 Documentation](https://v2.tauri.app/)
- [Tauri System Tray](https://v2.tauri.app/learn/system-tray/)
- [Tauri Notifications Plugin](https://v2.tauri.app/plugin/notification/)
- [Tauri Security Best Practices](https://v2.tauri.app/security/)
- [tauri-plugin-spotlight GitHub](https://github.com/zzzze/tauri-plugin-spotlight)
- [Tauri macOS Spotlight Example](https://github.com/ahkohd/tauri-macos-spotlight-example)

### React & Frontend
- [React 19 Release](https://react.dev/blog/2024/12/05/react-19)
- [Vite 7.0 Announcement](https://vite.dev/blog/announcing-vite7)
- [Tailwind CSS v4 Blog](https://tailwindcss.com/blog/tailwindcss-v4)
- [Tailwind CSS with Vite](https://tailwindcss.com/docs)
- [Zustand GitHub](https://github.com/pmndrs/zustand)
- [State Management in React (2026)](https://www.c-sharpcorner.com/article/state-management-in-react-2026-best-practices-tools-real-world-patterns/)
- [React State Management 2025: What You Actually Need](https://www.developerway.com/posts/react-state-management-2025)
- [Recharts GitHub](https://github.com/recharts/recharts)

### Node.js & Backend
- [Hono Documentation](https://hono.dev/docs/)
- [Vercel AI SDK 6](https://vercel.com/blog/ai-sdk-6)
- [Vercel AI SDK Documentation](https://ai-sdk.dev/docs/introduction)
- [MCP TypeScript SDK GitHub](https://github.com/modelcontextprotocol/typescript-sdk)
- [Building a TypeScript MCP Server](https://medium.com/@jageenshukla/building-a-typescript-mcp-server-a-guide-for-integrating-existing-services-5bde3fc13b23)

### Database & Vector Search
- [sqlite-vec GitHub](https://github.com/asg017/sqlite-vec)
- [Introducing sqlite-vec v0.1.0](https://alexgarcia.xyz/blog/2024/sqlite-vec-stable-release/index.html)
- [Using sqlite-vec in Rust](https://alexgarcia.xyz/sqlite-vec/rust.html)
- [SQLite Vector Search](https://www.sqlite.ai/sqlite-vector)

### Rust & System Integration
- [Tokio Documentation](https://tokio.rs/)
- [tokio-cron-scheduler on crates.io](https://crates.io/crates/tokio-cron-scheduler)
- [notify GitHub](https://github.com/notify-rs/notify)
- [Process Manager MCP (pm-mcp)](https://github.com/patrickjm/pm-mcp)

### MCP & Agent Runtime
- [Code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp)
- [The importance of Agent Harness in 2026](https://www.philschmid.de/agent-harness-2026)
- [Building an AI-Agent with TypeScript and MCP](https://hygraph.com/blog/build-ai-agent-with-typescript-and-mcp)
- [MCP-UI Documentation](https://mcpui.dev/)
- [Docker MCP Toolkit](https://www.docker.com/blog/mcp-toolkit-mcp-servers-that-just-work/)

### Dynamic UI & Rendering
- [Dynamic React: Rendering Remote JSX](https://dev.to/mirshahreza/dynamic-react-rendering-remote-jsx-without-a-build-step-2aal)
- [its-just-ui MCP Server](https://skywork.ai/skypage/en/its-just-ui-mcp-server-ai-co-pilot-react/1980832982031839232)

---

**Research complete. All major technology decisions documented with current 2026 versions and rationale.**
