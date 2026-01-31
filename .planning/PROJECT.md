# AIOS

## What This Is

A spotlight-style agent launcher and task manager for macOS. Instead of chatting with AI, you pop up a launcher (like Spotlight), fire off tasks to background agents, and get OS notifications when they're done. Agents work autonomously using MCP tools — executing code in Docker, controlling the host, searching the web, reading files — while a horizontal board UI lets you browse their outputs and manage active workers.

Built as a Tauri 2.0 app (Rust + React/TypeScript) with universal-agent-harness as the agent runtime and process-mcp for computer interaction.

## Core Value

Background agents that act relentlessly without human intervention — you launch a task, it gets done, you get notified.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Spotlight-style launcher for sending tasks to agents
- [ ] Background agent execution via universal-agent-harness
- [ ] Horizontal scrollable board UI (management board + task output boards)
- [ ] CMD+K search to jump between boards
- [ ] OS-level notifications on task completion and when agents need input
- [ ] Floating indicator after task dispatch (fades after ~5s)
- [ ] Agent management board (first board — shows all active/completed workers)
- [ ] Process-MCP integration in dual mode (Docker for sandboxed work, host for real system actions)
- [ ] Screenshot capture of current screen as context for agents
- [ ] Persistent agent memory (SQLite keyword search + vector semantic search)
- [ ] Periodic memory extraction from conversation history
- [ ] "Remember" tool for agents to explicitly store information
- [ ] Declarative UI rendering — agents write JSX with pre-built components, app renders live
- [ ] Settings UI generated from component library (agent writes JSX layout)
- [ ] Event hooks/triggers system (cron, webhooks, file watchers, OS events)
- [ ] Hook-triggered agents with preconfigured behavior
- [ ] Agent planning via task management tools (lightweight planning before execution)
- [ ] Agent notification/question UI — agents can ask user for input via notifications
- [ ] SQLite persistence for runs, messages, memory, hooks

### Out of Scope

- Full chat interface with conversation history — this is a launcher, not a chat app
- Mobile app — desktop-first (macOS)
- Multi-user / auth — single-user local app
- Custom LLM hosting — uses external providers via universal-agent-harness
- Real-time collaboration — solo tool

## Context

**Existing libraries (owned by user):**
- `universal-agent-harness` (~/dev/universal-agent-harness) — TypeScript agent orchestration framework. Tick-based execution, MCP integration, multi-provider LLM support, SQLite persistence, cost tracking. Used as the core agent runtime.
- `process-mcp` (~/dev/process-mcp) — MCP server for process execution. Host mode (sandboxed local execution) and Docker mode (containerized). TTY support, background processes, stdin/stdout streaming.

**Previous codebase (aios-chat):**
- Tauri 2.0 + React + TypeScript + Vite + Tailwind
- Had dynamic UI rendering via dynamic-ui-mcp (agents write TSX, frontend renders)
- Zustand for state management
- Hono node backend for AI SDK streaming
- Patterns worth carrying forward: dynamic UI rendering, theme system, Tauri IPC for native features

**Key architectural decisions from previous work:**
- Three-layer design works well (Frontend → Node backend → Rust/Tauri)
- Node backend needed because AI SDK and MCP clients require Node.js runtime
- SQLite via Tauri for persistence is solid

**New approach:**
- Replace custom AI integration with universal-agent-harness
- Replace ad-hoc MCP setup with harness's profile-based MCP configuration
- Spotlight launcher replaces full chat window
- Boards replace chat threads
- Background-first instead of conversation-first

## Constraints

- **Tech stack**: Tauri 2.0 (Rust) + React/TypeScript + Vite + Tailwind — proven in previous iteration
- **Agent runtime**: universal-agent-harness — no custom agent loop, use the library
- **Process execution**: process-mcp in both Docker and host modes
- **Platform**: macOS first (Tauri is cross-platform but notifications/spotlight behavior tuned for macOS)
- **Local-only**: All data stays on device, no cloud services except LLM API calls
- **Component library**: Pre-built React components that agents can compose via JSX for dynamic UIs

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| New repo, fresh start | Current codebase is messy, want clean architecture | — Pending |
| universal-agent-harness as runtime | Already built, handles agents/MCP/providers/persistence | — Pending |
| process-mcp dual mode | Docker for freedom, host for real system actions | — Pending |
| Spotlight launcher over chat window | Agents work in background, not conversational | — Pending |
| Horizontal board scroller | Infinite scroll for task outputs, CMD+K for navigation | — Pending |
| SQLite + vector DB for memory | Keyword (FTS5) + semantic search for agent recall | — Pending |
| All four trigger types in v1 | Cron, webhooks, file watchers, OS events for hooks | — Pending |

---
*Last updated: 2026-01-31 after initialization*
