# AIOS

A spotlight-style agent launcher and task manager for macOS. Pop up a launcher, fire off tasks to background AI agents, and get OS notifications when they're done. Agents work autonomously using MCP tools while a horizontal board UI lets you browse their outputs and manage active workers.

## Architecture

Built as a **Tauri 2.0** app with three layers:

- **Frontend** — React + TypeScript + Vite + Tailwind (spotlight launcher, board UI)
- **Backend** — Node.js service (agent orchestration, MCP client)
- **Native** — Rust/Tauri (system integration, SQLite persistence, notifications)

### Key Dependencies

- [`universal-agent-harness`](https://github.com/anonx3247/universal-agent-harness) — Agent runtime with tick-based execution, MCP integration, multi-provider LLM support
- [`process-mcp`](https://github.com/anonx3247/process-mcp) — MCP server for process execution (Docker sandboxed + host mode)

## Structure

```
apps/
  desktop/     # Tauri desktop app (React frontend + Rust backend)
  backend/     # Node.js backend service (TypeScript)
```

## Setup

```bash
pnpm install
```

## Features

- Spotlight-style launcher for dispatching tasks to agents
- Background agent execution — launch a task, get notified when it's done
- Horizontal scrollable board UI with CMD+K navigation
- OS-level notifications on completion or when agents need input
- Declarative UI rendering — agents write JSX, the app renders it live
- Persistent agent memory (SQLite FTS5 + vector semantic search)
- Event hooks/triggers (cron, webhooks, file watchers, OS events)
- Process execution via Docker (sandboxed) or host mode

## Design Principles

- **Background-first**: Agents work autonomously, not conversationally
- **Local-only**: All data stays on device (except LLM API calls)
- **macOS-first**: Notifications and launcher behavior tuned for macOS
