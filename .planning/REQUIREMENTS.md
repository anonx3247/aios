# Requirements: AIOS

**Defined:** 2026-01-31
**Core Value:** Background agents that act relentlessly without human intervention

## v1 Requirements

### Launcher

- [ ] **LNCH-01**: User can open spotlight launcher via global hotkey
- [ ] **LNCH-02**: User can type natural language task description and dispatch to agent
- [ ] **LNCH-03**: Floating indicator appears after task dispatch, fades after ~5s
- [ ] **LNCH-04**: Home screen (CMD+H) shows all active/completed agent progress indicators

### Agent Runtime

- [ ] **AGNT-01**: Agent runs execute in background via universal-agent-harness library
- [ ] **AGNT-02**: Agent has access to process-mcp in host mode for system actions
- [ ] **AGNT-03**: Agent uses lightweight planning before executing tasks
- [ ] **AGNT-04**: Multiple agents can run concurrently

### Memory

- [ ] **MEM-01**: All agent runs and messages persisted to SQLite (via harness)
- [ ] **MEM-02**: Agent has a "remember" tool that appends facts to MEMORY.md
- [ ] **MEM-03**: Agent has a "recall" tool that greps MEMORY.md for relevant memories

### Notifications

- [ ] **NOTF-01**: Native macOS notification when agent task completes
- [ ] **NOTF-02**: Native macOS notification when agent encounters an error

### Infrastructure

- [ ] **INFR-01**: Tauri 2.0 app shell with Rust backend
- [ ] **INFR-02**: React/TypeScript/Vite/Tailwind frontend
- [ ] **INFR-03**: Node.js backend (Hono) for agent orchestration via harness
- [ ] **INFR-04**: SQLite database for persistence
- [ ] **INFR-05**: Clean project structure with proper separation of concerns

## v2 Requirements

### Process Execution

- **PROC-01**: Docker mode for process-mcp (sandboxed execution)
- **PROC-02**: Dual mode switching (agent chooses Docker vs host)
- **PROC-03**: Screenshot capture of current screen as agent context

### UI

- **UI-01**: Horizontal scrollable boards for task outputs
- **UI-02**: CMD+K board search to jump between boards
- **UI-03**: Declarative UI rendering (agent-written JSX with pre-built components)
- **UI-04**: Settings UI generated from component library

### Agent Intelligence

- **INTL-01**: Agent input requests via notifications (agent asks user for input)
- **INTL-02**: Semantic vector search for memory (sqlite-vec)
- **INTL-03**: Auto memory extraction from conversation history

### Automation

- **AUTO-01**: Cron/scheduled agent triggers
- **AUTO-02**: Webhook-triggered agent runs
- **AUTO-03**: File watcher triggers
- **AUTO-04**: OS event triggers
- **AUTO-05**: Hook-triggered agents with preconfigured behavior

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full chat interface | This is a launcher, not a chat app — agents work in background |
| Mobile app | Desktop-first (macOS), cross-platform later |
| Multi-user / auth | Single-user local app |
| Custom LLM hosting | Uses external providers via universal-agent-harness |
| Real-time collaboration | Solo tool |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| INFR-01 | | Pending |
| INFR-02 | | Pending |
| INFR-03 | | Pending |
| INFR-04 | | Pending |
| INFR-05 | | Pending |
| LNCH-01 | | Pending |
| LNCH-02 | | Pending |
| LNCH-03 | | Pending |
| LNCH-04 | | Pending |
| AGNT-01 | | Pending |
| AGNT-02 | | Pending |
| AGNT-03 | | Pending |
| AGNT-04 | | Pending |
| MEM-01 | | Pending |
| MEM-02 | | Pending |
| MEM-03 | | Pending |
| NOTF-01 | | Pending |
| NOTF-02 | | Pending |

**Coverage:**
- v1 requirements: 18 total
- Mapped to phases: 0
- Unmapped: 18 ⚠️

---
*Requirements defined: 2026-01-31*
*Last updated: 2026-01-31 after initial definition*
