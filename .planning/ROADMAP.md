# Roadmap: AIOS

## Overview

AIOS delivers a spotlight-style agent launcher through eight foundational phases. We start by establishing the three-layer architecture (Tauri + Node + React), then build the launcher interface for task dispatch, integrate universal-agent-harness for background execution, add persistent memory for agent intelligence, and close with native notifications for user feedback. Each phase delivers a complete, verifiable capability that brings us closer to autonomous background agents.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Project Setup** - Clean project structure with proper separation of concerns
- [ ] **Phase 2: Tauri Shell** - Rust backend with SQLite persistence
- [ ] **Phase 3: Node Backend** - React frontend and Hono server
- [ ] **Phase 4: Launcher UI** - Spotlight-style task dispatch interface
- [ ] **Phase 5: Agent Runtime** - Background execution via universal-agent-harness
- [ ] **Phase 6: Agent Intelligence** - Planning and concurrent execution
- [ ] **Phase 7: Memory System** - Persistent memory with keyword search
- [ ] **Phase 8: Notifications** - Native macOS feedback on task completion

## Phase Details

### Phase 1: Project Setup
**Goal**: Clean project foundation with proper directory structure and tooling
**Depends on**: Nothing (first phase)
**Requirements**: INFR-05
**Success Criteria** (what must be TRUE):
  1. Project has clear directory structure separating frontend, backend, and Rust layers
  2. Development environment runs via nix-shell with all dependencies available
  3. TypeScript compilation and linting work across all layers
  4. Build commands execute without errors (pnpm install, pnpm dev, pnpm build)
**Plans**: 2 plans

Plans:
- [ ] 01-01-PLAN.md — Monorepo root, Tauri desktop app, Nix flake
- [ ] 01-02-PLAN.md — Backend (Hono), shared packages, TypeScript config, ESLint

### Phase 2: Tauri Shell
**Goal**: Operational Tauri app with SQLite database and IPC commands
**Depends on**: Phase 1
**Requirements**: INFR-01, INFR-04
**Success Criteria** (what must be TRUE):
  1. Tauri window opens and displays a placeholder UI
  2. SQLite database initializes on first launch with schema for runs and messages
  3. Tauri commands are callable from frontend via IPC (test command works)
  4. App can be built as production binary (pnpm tauri build succeeds)
**Plans**: TBD

Plans:
- [ ] 02-01: [TBD during planning]

### Phase 3: Node Backend
**Goal**: React frontend and Hono HTTP server operational
**Depends on**: Phase 2
**Requirements**: INFR-02, INFR-03
**Success Criteria** (what must be TRUE):
  1. React app renders in Tauri window with Tailwind styling
  2. Hono server starts on dynamic port and responds to health check endpoint
  3. Frontend can call Node backend HTTP API and receive responses
  4. Node backend sidecar starts automatically with Tauri app
  5. Sidecar process terminates cleanly when Tauri app closes (no orphans)
**Plans**: TBD

Plans:
- [ ] 03-01: [TBD during planning]

### Phase 4: Launcher UI
**Goal**: User can dispatch tasks via spotlight-style launcher
**Depends on**: Phase 3
**Requirements**: LNCH-01, LNCH-02, LNCH-03, LNCH-04
**Success Criteria** (what must be TRUE):
  1. User can open launcher via global hotkey (CMD+Shift+Space)
  2. User can type natural language task and press Enter to dispatch
  3. Floating success indicator appears after dispatch and fades after 5 seconds
  4. Home screen (CMD+H) shows all dispatched tasks with status
  5. Launcher closes after task dispatch and reopens instantly on next hotkey
**Plans**: TBD

Plans:
- [ ] 04-01: [TBD during planning]

### Phase 5: Agent Runtime
**Goal**: Agents execute tasks in background via universal-agent-harness
**Depends on**: Phase 4
**Requirements**: AGNT-01, AGNT-02
**Success Criteria** (what must be TRUE):
  1. Dispatched task creates agent run via universal-agent-harness
  2. Agent run executes in background (user can close launcher during execution)
  3. Agent has access to process-mcp in host mode for system actions
  4. Agent run completion updates task status in home screen
  5. Agent run messages persist to SQLite via harness
**Plans**: TBD

Plans:
- [ ] 05-01: [TBD during planning]

### Phase 6: Agent Intelligence
**Goal**: Agents plan before executing and support concurrent runs
**Depends on**: Phase 5
**Requirements**: AGNT-03, AGNT-04
**Success Criteria** (what must be TRUE):
  1. Agent uses lightweight planning tool before executing task actions
  2. Multiple tasks can be dispatched and run concurrently (3-5 max)
  3. Home screen shows progress for all active agents simultaneously
  4. Agent planning output is visible in task detail view
**Plans**: TBD

Plans:
- [ ] 06-01: [TBD during planning]

### Phase 7: Memory System
**Goal**: Agents have persistent memory accessible via tools
**Depends on**: Phase 6
**Requirements**: MEM-01, MEM-02, MEM-03
**Success Criteria** (what must be TRUE):
  1. All agent runs and messages persist to SQLite (via harness)
  2. Agent has "remember" tool that appends facts to MEMORY.md file
  3. Agent has "recall" tool that searches MEMORY.md for relevant memories
  4. Recall tool returns results from keyword search (grep-like)
  5. Memory persists across agent runs and app restarts
**Plans**: TBD

Plans:
- [ ] 07-01: [TBD during planning]

### Phase 8: Notifications
**Goal**: User receives native macOS notifications from agents
**Depends on**: Phase 7
**Requirements**: NOTF-01, NOTF-02
**Success Criteria** (what must be TRUE):
  1. User receives macOS notification when agent task completes successfully
  2. User receives macOS notification when agent encounters an error
  3. Clicking notification opens home screen to relevant task
  4. Notifications appear even when app is not in foreground
**Plans**: TBD

Plans:
- [ ] 08-01: [TBD during planning]

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Project Setup | 0/TBD | Not started | - |
| 2. Tauri Shell | 0/TBD | Not started | - |
| 3. Node Backend | 0/TBD | Not started | - |
| 4. Launcher UI | 0/TBD | Not started | - |
| 5. Agent Runtime | 0/TBD | Not started | - |
| 6. Agent Intelligence | 0/TBD | Not started | - |
| 7. Memory System | 0/TBD | Not started | - |
| 8. Notifications | 0/TBD | Not started | - |
