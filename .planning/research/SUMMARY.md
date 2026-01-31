# Project Research Summary

**Project:** AIOS - Spotlight-Style AI Agent Launcher
**Domain:** Desktop AI Agent Platform (macOS-first)
**Researched:** 2026-01-31
**Confidence:** HIGH

## Executive Summary

AIOS is a desktop AI agent launcher that positions uniquely between traditional launchers (Alfred, Raycast) and chat-based AI assistants (ChatGPT Desktop). Research confirms the core vision is sound and technically feasible using a proven three-layer architecture: React frontend (UI), Node.js backend (agent orchestration), and Rust/Tauri (native capabilities). The recommended approach builds on the existing aios-chat codebase, replacing chat-centric UX with a spotlight launcher and horizontal board system while integrating universal-agent-harness for agent runtime and process-mcp for sandboxed execution.

The technology stack is mature and well-supported in 2026. Tauri 2.0 provides stable desktop framework foundations, Vercel AI SDK 6 brings agent support with streaming, and sqlite-vec enables local vector search for agent memory. The three-layer architecture is necessary (not optional) because AI SDK and MCP clients require Node.js runtime - attempting to avoid the Node backend would force rewrites of critical dependencies. All recommended libraries have active maintenance and clear migration paths.

The highest risks cluster around sidecar lifecycle management (orphaned processes), dynamic UI security (sandbox escapes), and event hook reliability (silent failures). These are well-understood problems with proven mitigation strategies. The research flags two areas needing phase-specific investigation: universal-agent-harness integration details (user-owned library, no public docs) and macOS Spotlight integration APIs (private APIs, limited documentation). Overall confidence is HIGH - this is a buildable product with a clear path to differentiation through background agents, horizontal boards, and dual-mode execution.

## Key Findings

### Recommended Stack

AIOS requires a three-layer architecture separating concerns: React handles presentation, Node.js orchestrates agents and MCP connections, and Rust/Tauri provides native OS integration and persistence. This architecture is validated by aios-chat and mandated by runtime requirements - Vercel AI SDK and MCP clients only run in Node.js.

**Core technologies:**
- **Tauri 2.0**: Desktop framework with 35% YoY growth, mature plugin system, excellent security model. Cross-platform but optimize for macOS.
- **React 19 + Vite 7 + Tailwind 4**: Modern frontend stack. Vite 7 offers 45% faster cold starts, Tailwind 4 has 100x faster incremental builds. Use client-only React (no SSR).
- **Node Backend (Hono + AI SDK 6)**: Hono for HTTP server (ultrafast, ~30KB), AI SDK 6 for LLM streaming with v3 Language Model Spec and agent support. Mandatory for harness integration.
- **universal-agent-harness**: User's existing library for tick-based agent runtime, multi-provider LLM support, MCP profile management, SQLite persistence, cost tracking. Core runtime - DO NOT build custom agent loop.
- **process-mcp**: User's MCP server for process execution in dual modes (Docker sandboxed + host mode). TTY support, background processes, stdin/stdout streaming.
- **SQLite + rusqlite + sqlite-vec**: Embedded database with FTS5 for keyword search and sqlite-vec for vector similarity. Pre-v1 (v0.1.0) but production-ready, zero dependencies, SIMD-accelerated.
- **Zustand**: State management with 30% YoY growth, lightweight (~3KB), no boilerplate. Ideal for desktop apps where Redux is overkill.

**What NOT to use:**
- Next.js/Remix (SSR incompatible with Tauri)
- React Server Components (requires server environment)
- Electron (heavier than Tauri, bundles Chromium)
- vm2 for dynamic UI sandbox (CVE-2026-22709, critical escape vulnerability)
- Custom agent orchestration (use universal-agent-harness)

### Expected Features

User expectations in 2026 have shifted from "fast app search" to "autonomous agents with persistent memory and proactive behavior." Table stakes now include multi-LLM support, background execution, and OS notifications.

**Must have (table stakes):**
- **Spotlight-style launcher** with keyboard-first navigation (CMD+Space) - without this, it's not a launcher
- **Multi-LLM provider support** (OpenAI, Anthropic, Perplexity minimum) - 2026 baseline for AI tools
- **Background execution** with job queue and status tracking - differentiates from chat apps
- **Persistent memory** using structured state (not context window) - required for intelligent agents
- **OS notifications** for task completion and agent needs-input events
- **Web search integration** (Perplexity or similar) - users expect current information

**Should have (differentiators):**
- **Horizontal board UI** for multi-agent output (unique to AIOS, not chat bubbles)
- **Declarative UI rendering** using A2UI standard (agents return rich UIs: charts, forms, dashboards)
- **Event hooks/triggers** (cron, webhooks, file watchers) - enables proactive agents
- **Dual process execution** (Docker safe mode + host mode) - security + capability tradeoff
- **Multi-agent orchestration** with Agent-to-Agent (A2A) protocol for complex workflows

**Defer (v2+):**
- Agent marketplace (community-built agents)
- Memory contracts & governance (GDPR-style agent memory control)
- Computer use / desktop automation (UI control via Anthropic APIs)
- Local model support (Ollama integration)
- Cross-app workflow integration (Slack, Jira, Linear)

**Anti-features (explicitly avoid):**
- Full chat interface as primary UI (AIOS is launcher + boards, not ChatGPT clone)
- Chat history as memory model (use structured state-based memory)
- Unrestricted agent access (require sandboxing and permission model)
- Executable code in UI rendering (use A2UI declarative-only, no eval())

### Architecture Approach

AIOS uses a three-layer architecture with clear boundaries: Frontend (React) for UI and user interaction, Node Backend (Hono + harness) for agent orchestration and MCP integration, Rust Backend (Tauri) for SQLite persistence and native OS features. Communication flows Frontend → HTTP → Node → IPC → Rust, with Server-Sent Events (SSE) for real-time agent updates from Node to Frontend.

**Major components:**
1. **Spotlight Launcher** (React) - Captures user input, creates agent runs, triggers Node backend via HTTP POST
2. **Board Scroller** (React) - Horizontal CSS Grid layout displaying multiple agent outputs, scroll-snapped, keyboard navigation (CMD+K)
3. **Dynamic UI Renderer** (React) - Compiles agent-generated TSX at runtime using Sucrase, renders in sandboxed iframe for security
4. **Hono HTTP Server** (Node) - REST API + SSE endpoint, routes /api/runs/create, /api/events for streaming updates
5. **universal-agent-harness Integration** (Node) - Wraps harness API (createRun, run, getRun), manages background worker pool for parallel agents
6. **Event Hooks Manager** (Node) - Cron scheduler (node-cron), webhook receiver, file watcher (chokidar), OS event listener
7. **SQLite Database** (Rust) - Tables for runs, messages, memory (with FTS5 + sqlite-vec), hooks, executions. Accessed via Tauri commands.
8. **Tauri Commands** (Rust) - IPC interface for save_message, search_memory_keyword, search_memory_vector, capture_screenshot, show_notification

**Key patterns:**
- **Three-layer communication**: Frontend never talks directly to Rust for agent operations - always route through Node (harness requires Node runtime)
- **Event-driven updates**: Use SSE for real-time agent updates instead of polling (lower latency, native browser EventSource API)
- **Hybrid memory search**: Combine SQLite FTS5 (keyword) + sqlite-vec (semantic) using Reciprocal Rank Fusion for balanced ranking
- **Background worker pool**: Run multiple agents in parallel with max concurrency limit (3-5) and FIFO queue

### Critical Pitfalls

Research identified 13 domain-specific pitfalls. Top 5 for AIOS:

1. **Sidecar Process Orphaning** - Tauri doesn't auto-kill child processes on exit. Orphaned Node backend accumulates over time, draining battery. Prevention: Implement heartbeat + watchdog pattern (main app sends heartbeat every 5s, sidecar self-terminates after 15s without heartbeat). Add cleanup handlers in Rust Drop trait.

2. **Dynamic UI Sandbox Escape** - Using vm2 or new Function() to execute agent-generated JSX leads to critical CVE-2026-22709 vulnerability (CVSS 9.8). Agent could read API keys, write arbitrary files, spawn processes. Prevention: Use sandboxed iframe (no allow-same-origin) with allowlist of safe React APIs. NEVER use vm2 or eval() in 2026.

3. **Semantic Privilege Escalation** - Agent granted broad permissions becomes privilege escalation intermediary. User asks innocent question, agent uses file system + webhook tools to exfiltrate data beyond task scope. Prevention: High-risk tools (delete_file, execute_shell_command, docker_run) require explicit user confirmation via OS notification. Implement semantic scope analyzer to validate tool calls align with task intent.

4. **Event Hook Silent Failures** - Cron jobs, file watchers, webhooks fail silently with no error, no retry, no alert. User expects daily backup cron but system was down during window - no backup, no notification. Prevention: Track execution vs scheduled time, detect missed runs, implement webhook retry with exponential backoff (5 attempts max), add health checks for file watchers with test file pattern.

5. **Vector Embedding Drift on Model Upgrade** - Upgrade embedding model (text-embedding-3-small v1→v2) breaks all existing vectors, memory search returns wrong results. "Agent forgot everything." Prevention: Pin embedding model version explicitly, implement Drift-Adapter pattern (lightweight transformation layer recovers 95-99% recall without full re-encoding), monitor recall metrics and alert on degradation.

**Other critical pitfalls:**
- MCP command injection (43% of implementations vulnerable) - use parameterized execution, never string concatenation
- OS notification spam → user disables all notifications - implement priority system (CRITICAL/HIGH/MEDIUM/LOW) and rate limiting (max 3 per 10 min)
- SQLite FTS5 performance degradation at scale (100K+ entries) - use hybrid search (FTS5 narrows to 100 candidates, vector re-ranks)

## Implications for Roadmap

Based on research, suggested phase structure prioritizes establishing unique positioning (background agents, board UI) while meeting table stakes (launcher, AI, memory). Critical path: Foundation → Agent Runtime → Unique UX → Advanced Features.

### Phase 1: Foundation (Core Launcher + Agent Runtime)
**Rationale:** Establish technical foundation before building differentiators. Launcher UX validates product concept, agent runtime proves background execution works. These have no mutual dependencies and can progress in parallel tracks.

**Delivers:**
- Spotlight-style global hotkey (CMD+Space) with instant appearance
- Basic app/file search with fuzzy matching (<100ms response)
- Keyboard-first navigation (arrow keys, Enter, Esc)
- Node backend HTTP server (Hono on dynamic port to avoid conflicts)
- universal-agent-harness integration (createRun, run, getRun)
- Background worker pool for parallel agent execution (max 3 concurrent)
- SQLite schema with runs, messages tables (FTS5 setup deferred)

**Addresses features:**
- Spotlight launcher (table stakes)
- Background execution (core differentiator)
- Keyboard navigation (table stakes)

**Avoids pitfalls:**
- Port conflicts (use get-port for dynamic allocation)
- Sidecar orphaning (implement cleanup handlers from day 1)
- Cross-platform paths (use Tauri path APIs, not hardcoded ~/Library)

**Research flags:** Standard patterns, minimal research needed. Spotlight keyboard shortcut registration may need macOS-specific API investigation.

---

### Phase 2: Agent Capabilities (Memory + Multi-LLM + Notifications)
**Rationale:** Agent foundation from Phase 1 needs intelligence (memory) and user communication (notifications) to be useful. Multi-LLM support is table stakes. Memory is complex (FTS5 + vector) so build incrementally.

**Delivers:**
- Multi-LLM provider support (OpenAI, Anthropic via AI SDK adapters)
- Persistent memory with hybrid search (FTS5 keyword + sqlite-vec semantic)
- Embedding generation and storage (pin model version to avoid drift)
- OS notifications via tauri-plugin-notification (macOS native)
- Notification priority system (CRITICAL/HIGH/MEDIUM/LOW)
- Web search integration (Perplexity MCP or similar)

**Uses stack:**
- AI SDK 6 for multi-provider abstraction
- rusqlite with FTS5 extension for keyword search
- sqlite-vec for vector similarity (v0.1.0)
- tauri-plugin-notification for native OS alerts

**Implements architecture:**
- Hybrid memory search pattern (RRF merge of keyword + vector results)
- Tauri commands for memory operations (save_memory_entry, search_memory_keyword, search_memory_vector)

**Avoids pitfalls:**
- Vector embedding drift (pin model version, add monitoring)
- FTS5 performance at scale (optimize with porter tokenization, prefix indexing)
- Notification spam (priority system, rate limiting not yet enforced)

**Research flags:** MEDIUM - sqlite-vec integration with rusqlite needs documentation review. Embedding model selection (performance vs cost) may need benchmarking.

---

### Phase 3: Unique UX (Horizontal Boards + Dynamic UI)
**Rationale:** Differentiation from Raycast/Alfred requires unique visual paradigm. Horizontal boards provide persistent workspace for agent outputs. Dynamic UI lets agents return rich visualizations instead of text.

**Delivers:**
- Horizontal board scroller (CSS Grid, full-viewport panels, scroll-snap)
- Board management (create, close, navigate with CMD+K)
- Dynamic UI renderer with Sucrase compilation
- Sandboxed iframe for agent-generated components
- Component allowlist (React, Recharts, Lucide - NO process, fs, require)
- Real-time board updates via SSE (agent → Node → Frontend)

**Implements architecture:**
- Board Scroller component (React)
- Dynamic UI Renderer with security sandbox
- SSE event streaming from Node to Frontend

**Avoids pitfalls:**
- Dynamic UI sandbox escape (use iframe sandbox="allow-scripts", NO vm2)
- Global state mutations (use Zustand's set() with immutable updates)
- Tight coupling to MCP (abstract MCP details in Node, expose high-level API)

**Research flags:** LOW - Dynamic UI pattern proven in aios-chat. A2UI standard (declarative UI) may need specification review if adopting beyond basic TSX compilation.

---

### Phase 4: Proactive Agents (Event Hooks + Triggers)
**Rationale:** Background execution from Phase 1 is reactive (user invokes). Event hooks enable proactive agents (cron schedules, file changes, webhooks). This is a key differentiator vs chat-based assistants.

**Delivers:**
- Cron scheduler with execution tracking and missed run detection
- File watcher with health checks (chokidar + test file pattern)
- Webhook receiver with retry logic (exponential backoff, 5 attempts, DLQ)
- OS event triggers (system wake, network change - macOS-specific)
- Hook management UI (create, edit, disable, view execution history)

**Uses stack:**
- node-cron for cron scheduling
- chokidar for cross-platform file watching
- EventEmitter2 for internal hook routing
- notify crate (Rust) for FS event integration

**Avoids pitfalls:**
- Event hook silent failures (execution tracking, missed run alerts, retry logic)
- Cron has no execution guarantee (catch-up logic on app startup)
- File watcher events don't fire in git repos (health checks every 10 minutes)
- Webhook delivery failures (retry with exponential backoff, DLQ after 5 attempts)

**Research flags:** MEDIUM - OS event triggers (NSWorkspace notifications on macOS) need platform-specific research. Webhook security (signature verification) may need auth library selection.

---

### Phase 5: Advanced Security + Orchestration
**Rationale:** Multi-agent orchestration and dual-mode execution are complex features requiring solid foundation from Phases 1-4. Security hardening (permissions, audit logging) becomes critical as agent capabilities expand.

**Delivers:**
- Dual process execution (Docker sandboxed + host mode with explicit selection)
- Docker health check (verify Docker running before sandbox execution)
- Host mode confirmation UI (user approves before execution)
- Execution mode badge in board UI (Docker vs host indicator)
- Multi-agent orchestration (A2A protocol, context hand-off)
- High-risk tool confirmation (OS notifications for delete, execute, webhook)
- Semantic scope analyzer (validate tool calls match task intent)
- Audit logging (tool executions, file access, hook triggers)

**Implements architecture:**
- process-mcp dual mode integration
- Background worker coordination for multi-agent workflows
- Semantic privilege escalation prevention layer

**Avoids pitfalls:**
- Semantic privilege escalation (confirmation + scope validation)
- MCP command injection (parameterized execution enforced)
- Docker mode fallback to host without warning (explicit mode, no fallback)

**Research flags:** HIGH - Multi-agent orchestration (A2A protocol) is emerging standard (Salesforce/Google), needs protocol specification review. process-mcp dual mode configuration needs library API documentation. Semantic scope analysis requires LLM-based intent classification research.

---

### Phase Ordering Rationale

**Why this order:**
1. **Foundation first** - Can't build agents without launcher + runtime. Foundation has no UI dependencies beyond basic input.
2. **Memory before UX** - Agents need memory to be intelligent, regardless of UX paradigm. Memory is backend-heavy, can develop parallel to UI work.
3. **Unique UX after agent capabilities** - Boards display agent outputs, so agents must produce useful outputs first. Dynamic UI needs agent runtime to test with.
4. **Proactive features after reactive** - Event hooks trigger agents, so agent execution must be stable first. Hooks are independent of UI, can develop after boards.
5. **Advanced features last** - Multi-agent orchestration and security hardening require all prior phases stable. These are complex, high-risk features.

**Dependency patterns from architecture:**
- SQLite schema must exist before Tauri commands (Phase 1)
- Node backend must exist before harness integration (Phase 1)
- SSE infrastructure before dynamic UI (Phase 2 → Phase 3)
- Harness integration before event hooks (Phase 1 → Phase 4)
- Agent execution stable before multi-agent orchestration (Phase 1-3 → Phase 5)

**Pitfall avoidance:**
- Dynamic port allocation (Phase 1) prevents localhost conflicts
- Sidecar cleanup handlers (Phase 1) prevent orphaned processes
- Embedding model pinning (Phase 2) prevents drift on upgrades
- Sandbox iframe (Phase 3) prevents code execution escapes
- Execution tracking (Phase 4) prevents silent hook failures
- Explicit mode selection (Phase 5) prevents Docker fallback surprises

### Research Flags

**Phases likely needing deeper research during planning:**

- **Phase 1 (Spotlight Integration):** macOS global shortcut registration may conflict with system Spotlight. Need to research tauri-plugin-spotlight compatibility with Tauri 2.0 or implement custom using macOS Accessibility APIs. Keyboard shortcut best practices (CMD+Shift+Space vs Option+Space).

- **Phase 2 (Memory System):** sqlite-vec v0.1.0 is pre-v1 but production-ready. Need to validate rusqlite FFI integration patterns and monitor for breaking changes. Embedding model selection (OpenAI text-embedding-3-small vs Anthropic vs local) requires cost/performance benchmarking.

- **Phase 4 (OS Event Triggers):** macOS-specific events (system wake, network change, app activation) require NSWorkspace notifications or launchd patterns. No cross-platform abstraction exists - need macOS-specific implementation research.

- **Phase 5 (Multi-Agent Orchestration):** A2A protocol is emerging (Salesforce Agentforce, Google ADK) but not finalized. Need to research current specification status and decide whether to implement proprietary protocol or wait for standard. Agent state synchronization patterns need investigation.

- **Phase 5 (Semantic Scope Analysis):** Intent classification to validate tool calls requires LLM-based analysis or rule-based DSL. Need to research accuracy vs cost tradeoff and decide on implementation approach.

**Phases with standard patterns (skip research-phase):**

- **Phase 1 (Foundation):** Tauri window management, Hono HTTP server, SQLite schema - all well-documented with clear examples.

- **Phase 2 (Multi-LLM):** AI SDK 6 has mature provider adapters with official documentation. Straightforward integration.

- **Phase 3 (Horizontal Boards):** CSS Grid horizontal scroller is standard pattern (Linear, Height, Notion). No novel research needed.

- **Phase 3 (Dynamic UI):** Pattern proven in aios-chat codebase. Sucrase compilation + iframe sandbox is established approach.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All technologies verified as current 2026 versions with active maintenance. Tauri 2.0 stable, AI SDK 6 released, sqlite-vec v0.1.0 production-ready. Three-layer architecture validated by aios-chat codebase. |
| Features | MEDIUM-HIGH | Feature landscape well-researched with 50+ sources. Table stakes clear (launcher, multi-LLM, background agents, memory). Differentiators informed by Raycast, Apple Intelligence, agent framework trends. Some uncertainty on A2A protocol maturity. |
| Architecture | HIGH | Three-layer pattern proven in aios-chat. Component boundaries clear, data flows documented with 5 concrete examples. Technology choices align with runtime constraints (harness requires Node). Build order dependency graph validated. |
| Pitfalls | MEDIUM-HIGH | 13 pitfalls identified with 2026 security advisories, GitHub issues, production experiences. Sidecar orphaning, sandbox escapes, event hook failures well-documented. Some domain-specific areas (universal-agent-harness internals, process-mcp security) lack public docs. |

**Overall confidence:** HIGH

Research grounded in official documentation, recent 2026 sources, and existing aios-chat codebase. Stack is mature and well-supported. Architecture is proven. Features align with market expectations. Pitfalls have known mitigations.

### Gaps to Address

Areas where research was inconclusive or needs validation during implementation:

- **universal-agent-harness integration specifics** - User-owned library with no public documentation. Need to review source code for:
  - Memory leak patterns in tick-based execution loop
  - SQLite connection pooling configuration
  - Cost tracking accuracy and edge cases
  - MCP profile configuration format and mode switching
  - Progress streaming callback API for SSE integration

  **How to handle:** Phase 1 research task to audit harness codebase, document API surface, identify integration points. Coordinate with library author if questions arise.

- **process-mcp security boundaries** - Docker + host dual mode raises security questions:
  - Container breakout vectors via volume mounts or capabilities
  - TTY/stdin streaming vulnerabilities
  - Host mode validation and confirmation UX patterns

  **How to handle:** Phase 5 research task for security audit. Review Docker security best practices, test container escape scenarios, implement defense-in-depth (least privilege mounts, capability restrictions, user confirmation).

- **macOS Spotlight integration APIs** - Private or undocumented APIs:
  - Global shortcut registration without conflicts
  - Spotlight-style window appearance animation
  - Accessibility permissions required for system-wide keyboard hooks

  **How to handle:** Phase 1 research into tauri-plugin-spotlight source code and macOS Accessibility framework. May require fallback to simpler keyboard shortcut if Spotlight-level integration proves fragile.

- **A2UI protocol maturity** - Declarative UI standard is emerging (Google 2026):
  - Current specification status (draft vs stable)
  - Component catalog standardization
  - Security model for agent-generated UI declarations

  **How to handle:** Phase 3 decision point - implement basic TSX compilation first (proven in aios-chat), evaluate A2UI adoption if standard stabilizes. A2UI is enhancement, not blocker.

- **Embedding drift monitoring** - Detection and mitigation unclear:
  - What recall threshold indicates drift (90%? 95%?)
  - Drift-Adapter training requirements (sample size, compute cost)
  - Zero-downtime migration workflow

  **How to handle:** Phase 2 experimentation. Start with version pinning and manual upgrades. Implement drift monitoring (test queries, recall metrics). Defer Drift-Adapter to v2 unless model upgrade forced.

## Sources

### Primary (HIGH confidence)

**Official Documentation:**
- [Tauri 2.0 Documentation](https://v2.tauri.app/) - Desktop framework, IPC, plugins, security model
- [Vercel AI SDK 6](https://ai-sdk.dev/docs/introduction) - LLM streaming, multi-provider, agent support
- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25) - Protocol definition
- [SQLite FTS5 Extension](https://sqlite.org/fts5.html) - Full-text search documentation
- [sqlite-vec GitHub](https://github.com/asg017/sqlite-vec) - Vector search extension, Rust integration guide

**Codebase Context:**
- aios-chat codebase - Existing three-layer architecture, dynamic UI rendering patterns, Tauri IPC examples

**Recent Releases:**
- [React 19 Release](https://react.dev/blog/2024/12/05/react-19) - New features, server components clarification
- [Vite 7.0 Announcement](https://vite.dev/blog/announcing-vite7) - Performance improvements, Node version requirements
- [Tailwind CSS v4 Blog](https://tailwindcss.com/blog/tailwindcss-v4) - New features, Vite plugin

### Secondary (MEDIUM confidence)

**Security Research:**
- [CVE-2026-22709: Critical vm2 sandbox escape](https://www.endorlabs.com/learn/cve-2026-22709) - Vulnerability disclosure
- [OWASP AI Agent Security Top 10 2026](https://medium.com/@oracle_43885) - Security framework
- [Semantic privilege escalation threat](https://acuvity.ai/semantic-privilege-escalation) - Agent security analysis
- [MCP security risks and controls](https://www.redhat.com/en/blog/model-context-protocol-mcp) - Security assessment

**GitHub Issues:**
- [Tauri sidecar orphaning #1896](https://github.com/tauri-apps/tauri/issues/1896) - Known issue, workarounds
- [Node backend running in background #8689](https://github.com/tauri-apps/tauri/issues/8689) - Community discussion
- [File watcher events in git repos](https://github.com/sst/opencode/issues/5087) - Known limitation

**Industry Analysis:**
- [State Management in React 2026](https://www.developerway.com/posts/react-state-management-2025) - Zustand growth trends
- [Best AI Agent Frameworks in 2026](https://www.datacamp.com/blog/best-ai-agents) - Market landscape
- [Raycast AI Features](https://www.raycast.com/core-features/ai) - Competitive feature analysis
- [Apple Intelligence Overview](https://www.apple.com/apple-intelligence/) - OS-level AI direction

**Performance & Optimization:**
- [Hybrid FTS5 + vector search](https://alexgarcia.xyz/blog/2024/sqlite-vec-hybrid-search/) - Search pattern guide
- [Drift-Adapter for embedding upgrades](https://arxiv.org/html/2509.23471) - Academic paper
- [Webhook retry best practices](https://www.svix.com/resources/webhook-best-practices/retries/) - Implementation guide

### Tertiary (LOW confidence)

**Emerging Standards:**
- [A2UI Protocol Guide](https://dev.to/czmilo/the-a2ui-protocol-a-2026-complete-guide) - Community documentation (spec not finalized)
- [A2A Protocol for Multi-Agent](https://www.salesforce.com/blog/ai-agent-pitfalls/) - Mentioned but no public spec

**User-Owned Libraries (no public docs):**
- universal-agent-harness (~/dev/universal-agent-harness) - Need to audit source
- process-mcp (~/dev/process-mcp) - Need to review security model

**Platform-Specific (needs validation):**
- [tauri-plugin-spotlight](https://github.com/zzzze/tauri-plugin-spotlight) - Compatibility with Tauri 2.0 uncertain
- macOS NSWorkspace APIs - Private API documentation sparse

---

**Research completed:** 2026-01-31
**Ready for roadmap:** Yes

**Next steps:** Proceed to requirements definition using Phase 1-5 structure. Flag Phase 1 (Spotlight integration), Phase 4 (OS events), and Phase 5 (A2A protocol) for deeper research during planning.
