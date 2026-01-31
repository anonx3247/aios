# Architecture Patterns for AIOS Desktop Agent Launcher

**Domain:** Desktop AI agent launcher with background workers
**Researched:** 2026-01-31
**Confidence:** HIGH

## Executive Summary

AIOS requires a three-layer architecture separating presentation (React), orchestration (Node.js), and native capabilities (Rust/Tauri). This architecture is proven by the existing aios-chat codebase and aligns with Tauri 2.0's multi-process model. The key architectural challenge is coordinating background agent execution with UI updates while maintaining clear component boundaries.

**Recommendation:** Build on the proven three-layer pattern from aios-chat, replacing the chat-centric design with a launcher-centric architecture where agents run independently in the Node backend, communicating with the Rust layer for persistence and with the React layer for UI updates via events.

## Recommended Architecture

### High-Level System Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                        TAURI WINDOW LAYER                             │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │              React/TypeScript Frontend                          │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │  │
│  │  │  Spotlight   │  │   Board      │  │  Dynamic UI          │ │  │
│  │  │  Launcher    │  │   Scroller   │  │  Renderer            │ │  │
│  │  │  (CMD+Space) │  │  (Horizontal)│  │  (Agent-generated    │ │  │
│  │  │              │  │              │  │   JSX components)    │ │  │
│  │  └──────────────┘  └──────────────┘  └──────────────────────┘ │  │
│  │                                                                  │  │
│  │  ┌──────────────────────────────────────────────────────────┐  │  │
│  │  │          Zustand State Stores                             │  │  │
│  │  │  - Agent runs (active/completed)                          │  │  │
│  │  │  - Board state (current board, scroll position)           │  │  │
│  │  │  - Notifications (queued, shown)                          │  │  │
│  │  │  - Memory search results                                  │  │  │
│  │  └──────────────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                              │                                        │
│                              │ HTTP + Server-Sent Events (SSE)       │
│                              ▼                                        │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│                       NODE.JS SIDECAR (PORT 3001)                     │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │              Hono HTTP Server + EventEmitter                    │  │
│  │  Routes:                                                         │  │
│  │  - POST   /api/runs/create                                      │  │
│  │  - POST   /api/runs/:name/tick                                  │  │
│  │  - GET    /api/runs/:name                                       │  │
│  │  - GET    /api/runs                                             │  │
│  │  - GET    /api/events (SSE for agent updates)                   │  │
│  │  - POST   /api/memory/search                                    │  │
│  │  - POST   /api/hooks/register                                   │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                              │                                        │
│                              ▼                                        │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │        universal-agent-harness Integration                      │  │
│  │  - createRun() → Initialize agent with profile + problem       │  │
│  │  - run() → Execute tick-based agent loop                       │  │
│  │  - getRun() → Fetch run state                                  │  │
│  │  - Background worker pool for parallel agent execution         │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                              │                                        │
│                              ▼                                        │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │          MCP Server Connections (via harness)                   │  │
│  │  - process-mcp (host + docker modes)                           │  │
│  │  - dynamic-ui-mcp (UI generation)                              │  │
│  │  - filesystem-mcp (file access)                                │  │
│  │  - perplexity-mcp (web search)                                 │  │
│  │  - Custom MCP servers per profile                              │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                              │                                        │
│                              ▼                                        │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │              Event Hooks Manager                                │  │
│  │  - Cron scheduler (node-cron)                                  │  │
│  │  - Webhook receiver (Hono routes)                              │  │
│  │  - File watcher (chokidar)                                     │  │
│  │  - OS event listener (via Tauri IPC)                           │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                              │                                        │
│                              │ Tauri IPC Commands                    │
│                              ▼                                        │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│                       TAURI BACKEND (RUST)                            │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │              SQLite Database (via Tauri data dir)               │  │
│  │  Tables:                                                         │  │
│  │  - runs (id, name, status, created_at, completed_at)           │  │
│  │  - messages (run_id, role, content, timestamp)                 │  │
│  │  - memory_entries (id, content, keywords, timestamp)           │  │
│  │  - memory_vectors (id, entry_id, embedding_blob)               │  │
│  │  - hooks (id, type, config, enabled, last_triggered)           │  │
│  │                                                                  │  │
│  │  Extensions:                                                     │  │
│  │  - FTS5 (keyword search on memory_entries)                     │  │
│  │  - sqlite-vec (vector similarity search)                       │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                              │                                        │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │              Tauri Commands (IPC interface)                     │  │
│  │  - save_message(run_id, message)                               │  │
│  │  - get_run_messages(run_id)                                    │  │
│  │  - save_memory_entry(content, keywords, embedding)             │  │
│  │  - search_memory_keyword(query)                                │  │
│  │  - search_memory_vector(embedding, limit)                      │  │
│  │  - register_hook(type, config)                                 │  │
│  │  - list_hooks()                                                │  │
│  │  - capture_screenshot() → base64                               │  │
│  │  - show_notification(title, body)                              │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                              │                                        │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │              Native OS Integration                              │  │
│  │  - Screenshot capture (platform-specific)                      │  │
│  │  - Notification center (macOS UserNotifications)               │  │
│  │  - Global hotkey registration (CMD+Space)                      │  │
│  │  - File system watchers (notify crate)                         │  │
│  └────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

### Component Boundaries

| Component | Responsibility | Communicates With | Data Format |
|-----------|---------------|-------------------|-------------|
| **Spotlight Launcher** | Capture user input, trigger agent runs | Node backend (HTTP) | JSON task description |
| **Board Scroller** | Display active/completed runs horizontally | Zustand stores | React state |
| **Dynamic UI Renderer** | Compile & render agent-generated TSX | Node backend (SSE), harness output | TSX strings → React components |
| **Zustand Stores** | Frontend state management | All React components | JavaScript objects |
| **Hono Server** | HTTP API, SSE events, agent orchestration | Frontend (HTTP), Harness (library calls), Tauri (IPC) | JSON over HTTP, events via SSE |
| **universal-agent-harness** | Agent execution, MCP integration, tick loop | Hono server (library API), MCP servers (stdio/HTTP) | TypeScript library API |
| **MCP Servers** | Tool execution (filesystem, process, search) | Harness (JSON-RPC over stdio) | MCP protocol (JSON-RPC) |
| **Event Hooks Manager** | Schedule/trigger agent runs | Hono server (internal), Tauri (IPC for OS events) | Hook configs, cron patterns |
| **SQLite Database** | Persistent storage for runs, messages, memory | Tauri commands (SQL queries) | SQLite rows |
| **Tauri Commands** | IPC bridge for native features | Node backend (invoke), Rust impl | JSON arguments/returns |
| **Native OS Integration** | Screenshots, notifications, hotkeys | Tauri commands (Rust functions) | Platform-specific APIs |

## Data Flow Patterns

### Flow 1: Agent Run Creation (Spotlight → Background Execution)

```
1. User opens spotlight (CMD+Space)
   └─> React: Show launcher overlay

2. User types task + hits Enter
   └─> React: POST /api/runs/create { task, profile }
       └─> Node: createRun({ problemId, model, profile })
           └─> Harness: Initialize run in SQLite (via harness internal DB)
           └─> Harness: Load profile (prompt.md + settings.json)
           └─> Harness: Connect MCP servers per profile
       └─> Node: Start background worker
           └─> Worker: run({ runName, onMessage })
               └─> Harness: Tick-based execution loop
                   └─> LLM: Stream tool calls
                   └─> MCP: Execute tools
                   └─> Emit: SSE events (/api/events)
                       └─> React: Update Zustand store
                       └─> React: Render board updates
       └─> Node: Return { runId, status: "running" }
   └─> React: Close spotlight, show floating indicator (5s fade)
   └─> React: SSE connection for updates

3. On completion
   └─> Node: Emit "complete" event via SSE
       └─> React: Update run status in Zustand
   └─> Tauri: show_notification("Task Complete", summary)
       └─> macOS: Display notification banner
```

### Flow 2: Memory Storage & Retrieval

```
1. Agent uses "remember" tool
   └─> MCP: Tool execution via dynamic-ui-mcp
       └─> Node: Extract content + keywords
       └─> Node: Generate embedding (via embedding model API)
       └─> Tauri: save_memory_entry(content, keywords, embedding)
           └─> SQLite: INSERT into memory_entries
           └─> SQLite: INSERT into memory_vectors (vec_f32 blob)
           └─> FTS5: Index keywords

2. Agent queries memory
   └─> MCP: Tool call with query string
       └─> Node: Hybrid search strategy
           ├─> Tauri: search_memory_keyword(query)
           │   └─> SQLite FTS5: MATCH query
           └─> Node: Generate query embedding
               └─> Tauri: search_memory_vector(embedding, limit=10)
                   └─> sqlite-vec: KNN search
       └─> Node: Reciprocal Rank Fusion (RRF) merge
       └─> Return: Top 5 results to agent context
```

### Flow 3: Event Hook Trigger → Agent Run

```
1. Hook triggers (e.g., cron "0 9 * * *")
   └─> Node: Event Hooks Manager fires
       └─> Node: Lookup hook config in SQLite
       └─> Node: createRun({ problemId: hook.taskId, profile: hook.profile })
       └─> [Same flow as Flow 1 from step 2]

2. Webhook trigger (e.g., GitHub push)
   └─> External: POST /api/hooks/webhook/:hookId { payload }
       └─> Node: Verify signature (if configured)
       └─> Node: Extract payload data
       └─> Node: createRun({ problemId, profile, context: payload })

3. File watcher trigger
   └─> Tauri: File modified event (via notify crate)
       └─> Tauri: Invoke Node backend via fetch
           └─> Node: POST /api/hooks/file-changed { path, event }
               └─> Node: createRun({ problemId, context: filePath })

4. OS event trigger (e.g., wake from sleep)
   └─> Tauri: OS event listener fires
       └─> Tauri: Invoke Node backend
           └─> Node: createRun for scheduled hook
```

### Flow 4: Dynamic UI Rendering (Agent → Frontend)

```
1. Agent calls renderCustom tool
   └─> MCP: dynamic-ui-mcp tool execution
       └─> Tool: Validate TSX source
       └─> Tool: Return { component: "<ComponentSource>" }
   └─> Harness: Include in tool result message
   └─> Node: Emit SSE event { type: "tool_result", component }
       └─> React: Receive via SSE listener
           └─> DynamicUIRenderer: Compile TSX string
               └─> Sucrase: Transform TSX → JS
               └─> React.createElement: Instantiate component
               └─> React: Render in board content area
```

### Flow 5: Screenshot Context Capture

```
1. User triggers screenshot in launcher (or agent requests)
   └─> React: Request screenshot
       └─> Tauri: capture_screenshot()
           └─> macOS: CGWindowListCreateImage (entire screen)
           └─> Rust: Encode as PNG → base64
       └─> Tauri: Return base64 string
   └─> React: Include in agent context
       └─> Node: POST /api/runs/create { task, screenshot }
           └─> Harness: Add screenshot to initial messages
               └─> LLM: Vision model analyzes screenshot
```

## Technology Stack per Layer

### Frontend (React/TypeScript)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| React | 18.x | UI framework | Component model, ecosystem |
| TypeScript | 5.x | Type safety | Catch errors at compile time |
| Vite | 5.x | Build tool | Fast HMR, modern defaults |
| Tailwind CSS | 3.x | Styling | Utility-first, consistent design |
| Zustand | 4.x | State management | Minimal boilerplate, 30%+ YoY growth (2026) |
| Sucrase | 3.x | TSX compilation | Fast, browser-compatible for dynamic UI |

### Node Backend (Node.js/TypeScript)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Hono | 4.x | HTTP server | Fast, Vite-like DX, edge-compatible |
| universal-agent-harness | Latest | Agent runtime | Existing library, tick-based execution |
| process-mcp | Latest | Process execution | Host + Docker modes for sandboxing |
| node-cron | 3.x | Cron scheduling | Standard cron syntax, lightweight |
| chokidar | 3.x | File watching | Cross-platform, reliable |
| EventEmitter2 | 6.x | Internal events | Wildcard support for hook routing |

### Tauri Backend (Rust)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Tauri | 2.x | Desktop framework | Cross-platform, security-first |
| SQLite | 3.x | Database | Embedded, serverless, proven |
| rusqlite | 0.31.x | SQLite bindings | Safe, ergonomic Rust API |
| sqlite-vec | Latest | Vector search | SIMD-accelerated, no dependencies |
| tokio | 1.x | Async runtime | Standard for async Rust |
| serde | 1.x | Serialization | JSON IPC with frontend |
| notify | 6.x | File watching | Cross-platform FS events |
| screenshots | 0.6.x | Screen capture | Cross-platform screenshots |

## Patterns to Follow

### Pattern 1: Three-Layer Communication (Frontend ↔ Node ↔ Rust)

**What:** Frontend never talks directly to Rust for agent operations. Always route through Node backend.

**When:** Agent execution, MCP tool calls, dynamic UI generation.

**Why:**
- Node.js runtime required for universal-agent-harness and MCP clients
- Keeps agent orchestration logic in one place
- Rust layer focuses on native features and persistence

**Example:**
```typescript
// GOOD: Frontend → Node → Harness
async function createAgentRun(task: string) {
  const response = await fetch('http://localhost:3001/api/runs/create', {
    method: 'POST',
    body: JSON.stringify({ task, profile: 'default' })
  });
  return response.json();
}

// BAD: Frontend → Rust → Node (complicates flow)
// Don't use Tauri commands for agent operations
```

### Pattern 2: Event-Driven Updates (SSE for Real-Time)

**What:** Use Server-Sent Events (SSE) for pushing agent updates to frontend instead of polling.

**When:** Agent tick execution, tool results, status changes, completion events.

**Why:**
- Lower latency than polling
- Native browser API (EventSource)
- One-way server → client is sufficient (no WebSocket complexity)

**Example:**
```typescript
// Node backend: Emit events during agent execution
await run({
  runName,
  onMessage: (msg) => {
    sseManager.emit('agent-message', { runName, message: msg });
  }
});

// Frontend: Subscribe to SSE stream
const eventSource = new EventSource('http://localhost:3001/api/events');
eventSource.addEventListener('agent-message', (event) => {
  const { runName, message } = JSON.parse(event.data);
  agentStore.addMessage(runName, message);
});
```

### Pattern 3: Hybrid Memory Search (Keyword + Vector)

**What:** Combine SQLite FTS5 (keyword) and sqlite-vec (semantic) using Reciprocal Rank Fusion.

**When:** Agent queries memory or user searches past runs.

**Why:**
- FTS5 catches exact matches and domain terms
- Vector search finds semantically similar content
- RRF provides balanced ranking without tuning weights

**Example:**
```typescript
async function hybridSearch(query: string, limit: number = 5) {
  // Keyword search via FTS5
  const keywordResults = await invoke('search_memory_keyword', { query });

  // Vector search via sqlite-vec
  const embedding = await generateEmbedding(query);
  const vectorResults = await invoke('search_memory_vector', { embedding, limit: 20 });

  // Reciprocal Rank Fusion
  return reciprocalRankFusion([keywordResults, vectorResults], limit);
}
```

### Pattern 4: Hook Registration with Validation

**What:** Validate hook configurations at registration time, fail fast.

**When:** User creates cron, webhook, file watcher, or OS event hooks.

**Why:**
- Invalid cron patterns fail silently at runtime
- Webhook signatures prevent unauthorized triggers
- File paths must exist or registration is meaningless

**Example:**
```typescript
async function registerHook(config: HookConfig) {
  // Validate before saving
  switch (config.type) {
    case 'cron':
      if (!cronParser.parseExpression(config.pattern)) {
        throw new Error('Invalid cron pattern');
      }
      break;
    case 'file':
      const exists = await fs.pathExists(config.path);
      if (!exists) {
        throw new Error('File path does not exist');
      }
      break;
  }

  // Save to SQLite
  await invoke('register_hook', { config });

  // Activate immediately
  hookManager.activate(config);
}
```

### Pattern 5: Background Worker Pool (Parallel Agents)

**What:** Run multiple agents in parallel using a worker pool pattern.

**When:** User triggers multiple tasks or hooks fire simultaneously.

**Why:**
- Agents are CPU/IO bound (waiting for LLM responses)
- Pool limits resource usage (max concurrent agents)
- Queue ensures FIFO fairness

**Example:**
```typescript
class AgentWorkerPool {
  private queue: RunRequest[] = [];
  private active: Set<string> = new Set();
  private maxConcurrent = 3;

  async enqueue(runName: string) {
    this.queue.push({ runName });
    this.processQueue();
  }

  private async processQueue() {
    while (this.queue.length > 0 && this.active.size < this.maxConcurrent) {
      const { runName } = this.queue.shift()!;
      this.active.add(runName);

      // Run in background
      run({ runName, onMessage: this.handleMessage })
        .finally(() => {
          this.active.delete(runName);
          this.processQueue(); // Process next
        });
    }
  }
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Mixing Persistence Layers

**What:** Storing some data in Tauri SQLite and other data in harness SQLite.

**Why bad:** Data fragmentation, unclear source of truth, complex migrations.

**Instead:**
- **Harness SQLite**: Temporary run state, messages during execution
- **Tauri SQLite**: Long-term storage (runs, memory, hooks)
- Sync from harness → Tauri on tick completion

### Anti-Pattern 2: Blocking Tauri Commands

**What:** Long-running operations (agent ticks, LLM calls) in Tauri command handlers.

**Why bad:** Freezes frontend IPC, blocks other commands, poor UX.

**Instead:** Always run agents in Node backend, use async Tauri commands only for quick queries.

### Anti-Pattern 3: Polling for Agent Updates

**What:** Frontend repeatedly calling GET /api/runs/:name to check status.

**Why bad:** Network overhead, latency, server load, battery drain.

**Instead:** SSE stream for real-time updates, frontend subscribes once.

### Anti-Pattern 4: Global State Mutations

**What:** Directly mutating Zustand state from SSE callbacks.

**Why bad:** React doesn't detect mutations, causes stale UI.

**Instead:** Use Zustand's `set()` with immutable updates.

```typescript
// BAD
eventSource.onmessage = (event) => {
  agentStore.runs[runName].status = 'complete'; // Mutation
};

// GOOD
eventSource.onmessage = (event) => {
  agentStore.set((state) => ({
    runs: {
      ...state.runs,
      [runName]: { ...state.runs[runName], status: 'complete' }
    }
  }));
};
```

### Anti-Pattern 5: Tight Coupling to MCP Specifics

**What:** Frontend/React code with hardcoded MCP tool names or schemas.

**Why bad:** Breaks when MCP servers change, limits flexibility.

**Instead:** Node backend abstracts MCP details, exposes high-level API to frontend.

## Scalability Considerations

| Concern | At 10 Runs | At 100 Runs | At 1000 Runs |
|---------|-----------|-------------|--------------|
| **SQLite Database** | No issues | FTS5 + indexes handle well | May need VACUUM, consider archival |
| **Memory Search** | Instant | Sub-100ms with sqlite-vec | Index vector table, limit result sets |
| **Agent Workers** | All concurrent | Pool with max 3-5 | Queue + prioritization, LRU cleanup |
| **SSE Connections** | One per frontend | One per frontend (unchanged) | Same (single user app) |
| **MCP Servers** | Start on demand | Reuse connections | Connection pooling, timeout cleanup |
| **Dynamic UI Compilation** | Compile on render | Cache compiled components | LRU cache with size limit (50 components) |

## Build Order (Dependency Graph)

### Phase 1: Foundation (No dependencies)

1. **Tauri Shell** - Empty Tauri app with window management
2. **SQLite Schema** - Database tables, FTS5 setup, migrations
3. **Node Backend Shell** - Hono server, basic routes, health check

### Phase 2: Core Communication (Depends on Phase 1)

4. **Tauri Commands** - IPC interface for SQLite operations
5. **HTTP Client** - Frontend utilities for Node backend
6. **SSE Infrastructure** - Event emitter (Node), EventSource (React)

### Phase 3: Agent Runtime (Depends on Phase 2)

7. **Harness Integration** - Wrap universal-agent-harness API
8. **MCP Server Setup** - Configure process-mcp, dynamic-ui-mcp
9. **Background Workers** - Worker pool for parallel agent execution

### Phase 4: UI Components (Depends on Phase 3)

10. **Spotlight Launcher** - Input capture, run creation
11. **Board Scroller** - Horizontal layout, run display
12. **Dynamic UI Renderer** - TSX compilation, component instantiation

### Phase 5: Features (Depends on Phase 4)

13. **Memory System** - Vector embeddings, hybrid search
14. **Event Hooks** - Cron, webhooks, file watchers, OS events
15. **Notifications** - OS notifications on completion

### Dependency Rationale

- **SQLite before Tauri Commands**: Schema must exist before queries
- **Node backend before harness**: HTTP server must exist to expose agent API
- **SSE before workers**: Workers emit events, need transport ready
- **Harness before UI**: UI displays agent output, needs data source
- **Board scroller before dynamic UI**: Container must exist for rendered components
- **Memory before hooks**: Hooks may query memory, needs to be ready

## Critical Build Constraints

1. **Tauri sidecar bundling**: Node backend must be compiled to standalone binary (use `pkg` or `bun build --compile`)
2. **sqlite-vec loading**: Extension must be bundled with app or loaded from known path
3. **MCP server paths**: Absolute paths required in production (no `npx -y` shortcuts)
4. **Screenshot permissions**: macOS requires Screen Recording permission in Info.plist
5. **Notification permissions**: macOS requires Notification Center permission

## Sources

### Tauri Architecture & Sidecars
- [Embedding External Binaries | Tauri](https://v2.tauri.app/develop/sidecar/)
- [Process Model | Tauri](https://v2.tauri.app/concept/process-model/)
- [Node.js as a sidecar | Tauri](https://v2.tauri.app/learn/sidecar-nodejs/)
- [Adding Node.js server to Tauri App as a sidecar - DEV Community](https://dev.to/zaid_sunasra/adding-nodejs-server-to-tauri-app-as-a-sidecar-509j)

### IPC & Communication
- [Inter-Process Communication | Tauri](https://v2.tauri.app/concept/inter-process-communication/)
- [GitHub - MatsDK/TauRPC: Typesafe IPC layer for Tauri applications](https://github.com/MatsDK/TauRPC)

### SQLite & Vector Search
- [Hybrid full-text search and vector search with SQLite | Alex Garcia's Blog](https://alexgarcia.xyz/blog/2024/sqlite-vec-hybrid-search/index.html)
- [GitHub - asg017/sqlite-vec: A vector search SQLite extension that runs anywhere!](https://github.com/asg017/sqlite-vec)
- [SQLite FTS5 Extension](https://sqlite.org/fts5.html)

### MCP Protocol
- [Specification - Model Context Protocol](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP Architecture: Components, Lifecycle & Client-Server Tutorial | Obot AI](https://obot.ai/resources/learning-center/mcp-architecture/)

### Event-Driven Architecture
- [Event Loop - Agent Development Kit](https://google.github.io/adk-docs/runtime/event-loop/)
- [Loop agents - Agent Development Kit](https://google.github.io/adk-docs/agents/workflow-agents/loop-agents/)

### State Management
- [5 React State Management Tools Developers Actually Use in 2025 | Syncfusion Blogs](https://www.syncfusion.com/blogs/post/react-state-management-libraries)
- [GitHub - pmndrs/zustand: Bear necessities for state management in React](https://github.com/pmndrs/zustand)

### Desktop App Patterns
- [Architecture Diagrams in System Design - GeeksforGeeks](https://www.geeksforgeeks.org/system-design/how-to-draw-architecture-diagrams/)
- [Data flow diagram: Components, purpose, and how to create](https://www.rudderstack.com/data-flow-diagram/)
