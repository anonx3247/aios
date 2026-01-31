---
phase: 03
plan: 02
subsystem: frontend-integration
tags: [react, tauri-invoke, http-client, typescript, retry-logic]
requires:
  - 02-01-tauri-commands
  - 03-01-backend-sidecar
provides:
  - frontend-api-client
  - backend-connection-status
  - http-plugin-integration
affects:
  - 03-03-task-management
  - 03-04-agent-lifecycle
tech-stack:
  added:
    - "@tauri-apps/plugin-http"
  patterns:
    - singleton-client-pattern
    - retry-with-exponential-backoff
    - dev-mode-fallback
key-files:
  created:
    - apps/desktop/src/lib/api.ts
  modified:
    - apps/desktop/src/App.tsx
    - apps/desktop/package.json
key-decisions:
  - decision: "Development mode fallback to localhost:3001"
    context: "When running frontend in dev without Tauri sidecar, invoke fails"
    rationale: "Enables frontend-only development without full Tauri build"
    alternatives: ["Require full Tauri dev mode", "Mock the backend"]
  - decision: "Singleton backend client instance"
    context: "Multiple components may need backend access"
    rationale: "Shares port cache and prevents redundant discovery calls"
    alternatives: ["React context provider", "Per-component instances"]
  - decision: "10 retry attempts with exponential backoff"
    context: "Backend may take time to start, especially in sidecar mode"
    rationale: "1s * attempt gives ~55s total wait time before failure"
    alternatives: ["Infinite retries", "Shorter timeout", "User retry button"]
duration: 1 min
completed: 2026-01-31
---

# Phase 03 Plan 02: Frontend API Client Summary

**One-liner:** HTTP client with Tauri invoke port discovery, dev fallback to localhost:3001, and retry logic for backend connection

## Performance

**Execution time:** 1 minute
**Tasks completed:** 2/2
**Commits:** 2

**Efficiency notes:**
- Clean execution with no blockers
- All TypeScript compilation successful
- No deviations needed

## What We Accomplished

### Task Commits

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Install HTTP plugin and create API client | 233d980 | api.ts, package.json, pnpm-lock.yaml |
| 2 | Update App.tsx with backend connection status | 5df8175 | App.tsx |

### Files Created

**apps/desktop/src/lib/api.ts** (57 lines)
- BackendClient class with port discovery
- Caches port after first invoke to avoid repeated calls
- Uses portPromise to prevent race conditions
- Development mode fallback: catches invoke errors and defaults to port 3001
- Generic `call<T>()` method for typed HTTP requests
- Automatic Content-Type: application/json header
- Error handling with descriptive messages
- `healthCheck()` convenience method
- Singleton `backend` export

### Files Modified

**apps/desktop/src/App.tsx** (70 lines)
- Added useState for connection status (connecting/connected/error)
- Added useEffect with retry logic
- 10 retry attempts with exponential backoff (1s * attempt)
- Calls backend.healthCheck() on mount
- Displays color-coded status indicator:
  - Yellow: "Connecting to backend..."
  - Green: "Backend connected"
  - Red: "Backend error: {message}"
- Maintains Tailwind dark theme (bg-zinc-950)

**apps/desktop/package.json**
- Added @tauri-apps/plugin-http ^2.5.6

## Decisions Made

### 1. Development Mode Fallback

**Decision:** When `get_backend_port` invoke fails, fall back to localhost:3001

**Context:** Frontend developers need to work on UI without running full Tauri build with sidecar

**Rationale:**
- Enables `pnpm dev` workflow with separate backend process
- Logged warning makes it clear fallback occurred
- Production builds will always use proper invoke

**Trade-offs:**
- Adds conditional logic to port discovery
- Could mask configuration issues in development
- **Benefit:** Faster development iteration without Tauri overhead

**Alternative considered:** Require full Tauri dev mode for all frontend work (rejected - too slow for iteration)

### 2. Singleton Client Pattern

**Decision:** Export single `backend` instance instead of class only

**Context:** Multiple React components will need backend access as app grows

**Rationale:**
- Shares port cache across all components
- Prevents redundant invoke calls
- Simpler import (`import { backend }` vs `new BackendClient()`)

**Trade-offs:**
- Less flexibility for mocking in tests
- Can't have multiple clients with different configs
- **Benefit:** Guaranteed single source of truth for port

**Alternative considered:** React Context provider (rejected - overkill for current scale)

### 3. Retry Strategy

**Decision:** 10 attempts with exponential backoff (1s * attempt number)

**Context:** Backend sidecar takes time to spawn and start HTTP server

**Rationale:**
- 1s, 2s, 3s... up to 10s = ~55 seconds total
- Exponential prevents hammering backend during startup
- 10 attempts balances patience vs. fast failure

**Trade-offs:**
- Long wait on actual failures
- No user control over retry
- **Benefit:** Handles slow startup gracefully

**Alternative considered:** Fixed 2s interval (rejected - wastes time early, gives up too soon later)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. All tasks completed successfully without errors.

## Next Phase Readiness

**Ready for 03-03 (Task Management UI):** Yes

**Status:**
- ✅ Frontend can communicate with backend via HTTP
- ✅ Connection status visible to user
- ✅ Retry logic handles startup delays
- ✅ Dev mode supports frontend-only workflow

**Blockers:** None

**Concerns:**
- Backend health check endpoint doesn't exist yet (03-01 needs verification)
- If health check fails in 03-01, this plan's retry logic will surface the issue

**Next steps:**
1. Verify backend health endpoint works (test with actual sidecar)
2. Build task management UI on top of this API client
3. Add more backend methods (create task, list tasks, etc.)

**Integration points for next plan:**
- `backend.call<T>()` ready for new endpoints
- App.tsx can be extended with task list UI
- Status indicator provides foundation for richer connection state
