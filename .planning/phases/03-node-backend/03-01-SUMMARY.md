---
phase: 03-node-backend
plan: 01
subsystem: backend-runtime
tags: [hono, node, sidecar, pkg, binary-packaging]
requires: [02-04]
provides: [sidecar-ready-backend, binary-build-tooling]
affects: [03-02, 03-03, 03-04]
tech-stack:
  added: [esbuild, @yao-pkg/pkg]
  patterns: [dynamic-port-allocation, stdout-ipc, graceful-shutdown]
key-files:
  created:
    - apps/backend/scripts/bundle.mjs
    - apps/backend/scripts/build-binaries.mjs
  modified:
    - apps/backend/src/index.ts
    - apps/backend/package.json
key-decisions:
  - decision: Use esbuild to bundle ESM to CommonJS before pkg compilation
    rationale: pkg doesn't fully support ESM modules; bundling ensures compatibility
    impact: Reliable binary builds across platforms
  - decision: Dynamic port allocation (port 0) as default when PORT env var not set
    rationale: Enables multiple backend instances and OS-level port management
    impact: Backend can run as Tauri sidecar without port conflicts
  - decision: Structured stdout logging (BACKEND_PORT:{port}) for Tauri parsing
    rationale: Tauri sidecar needs to extract port from subprocess output
    impact: Tauri frontend can discover backend port programmatically
  - decision: Human-readable logs to stderr, structured data to stdout
    rationale: Separates machine-parseable IPC from developer-friendly diagnostics
    impact: Clean separation of concerns for logging
  - decision: Tauri target triple naming for binaries (backend-aarch64-apple-darwin)
    rationale: Tauri expects platform-specific binary naming convention
    impact: Seamless integration with Tauri sidecar configuration
metrics:
  duration: 6m 15s
  completed: 2026-01-31
---

# Phase 3 Plan 1: Backend Sidecar Enhancement Summary

**One-liner:** Dynamic port Hono server with stdout IPC, graceful shutdown, and esbuild+pkg binary tooling for Tauri sidecar deployment

## Performance

**Total duration:** 6 minutes 15 seconds
**Tasks:** 2/2 completed
**Commits:** 2 task commits + 1 metadata commit

## What Was Accomplished

Enhanced the Hono backend server to operate as a Tauri sidecar process with dynamic port allocation, structured stdout communication, graceful shutdown handling, and cross-platform binary compilation tooling.

### Key Capabilities Delivered

1. **Dynamic Port Allocation**
   - Server defaults to port 0 (OS-assigned) when PORT env var not set
   - Enables running multiple instances without port conflicts
   - Supports explicit port specification via environment variable

2. **Structured Stdout IPC**
   - Logs `BACKEND_PORT:{port}` to stdout for Tauri parsing
   - Human-readable messages routed to stderr
   - Clean separation between machine IPC and developer diagnostics

3. **Enhanced Health Endpoint**
   - Returns status, app name, timestamp, and uptime
   - Enables frontend health monitoring and uptime tracking

4. **Graceful Shutdown**
   - SIGINT and SIGTERM handlers for clean process termination
   - Server.close() called before process.exit(0)
   - Prevents resource leaks and connection drops

5. **Binary Build Tooling**
   - esbuild bundles ESM to CommonJS for pkg compatibility
   - pkg compiles to platform-specific binaries
   - Tauri target triple naming (backend-aarch64-apple-darwin)
   - Output to apps/desktop/src-tauri/binaries/
   - Executable permissions automatically set (755)

## Task Commits

| Task | Commit | Description | Files |
|------|--------|-------------|-------|
| 1 | 213927a | Enhanced Hono server for sidecar operation | apps/backend/src/index.ts |
| 2 | cc0e1bd | Added pkg build tooling for platform binaries | apps/backend/package.json, apps/backend/scripts/build-binaries.mjs, apps/backend/scripts/bundle.mjs |

## Files Created

- `apps/backend/scripts/bundle.mjs` - esbuild bundler for ESM to CommonJS conversion
- `apps/backend/scripts/build-binaries.mjs` - pkg compiler for platform binaries with Tauri naming

## Files Modified

- `apps/backend/src/index.ts` - Dynamic port, stdout logging, graceful shutdown, enriched /health endpoint
- `apps/backend/package.json` - Added esbuild and @yao-pkg/pkg deps, build:bundle and build:binary scripts

## Decisions Made

### 1. esbuild + pkg Two-Step Build
**Context:** pkg doesn't fully support ESM modules (ERR_REQUIRE_ESM errors)
**Decision:** Use esbuild to bundle ESM to CommonJS, then pkg to create binary
**Alternatives Considered:**
- Change entire project to CommonJS (breaks modern Node.js practices)
- Use different packager like ncc or nexe (less mature than pkg)
**Impact:** Reliable binary builds, maintains ESM for development, single CommonJS bundle for production

### 2. Dynamic Port as Default
**Context:** Backend needs to run as Tauri sidecar, may have multiple instances
**Decision:** Default to port 0 (OS-assigned) when PORT env var not set
**Alternatives Considered:**
- Hardcode port 3001 (requires configuration, port conflicts)
- Require PORT env var (less ergonomic for development)
**Impact:** Zero-config operation, eliminates port conflicts, OS-level port management

### 3. Stdout for Structured IPC, Stderr for Human Logs
**Context:** Tauri needs to parse backend port from subprocess output
**Decision:** `console.log()` for machine-readable data (BACKEND_PORT:), `console.error()` for human messages
**Alternatives Considered:**
- All logging to stdout (Tauri parsing would be fragile)
- File-based IPC (adds complexity, filesystem dependencies)
**Impact:** Clean IPC contract, preserves developer-friendly logging

### 4. Tauri Target Triple Naming
**Context:** Tauri expects binaries named with platform-specific target triples
**Decision:** Output `backend-aarch64-apple-darwin` (matches Rust target naming)
**Alternatives Considered:**
- Generic naming like backend-macos-arm64 (doesn't match Tauri conventions)
- Platform detection at runtime (adds complexity)
**Impact:** Seamless Tauri sidecar configuration, follows Rust ecosystem conventions

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added esbuild bundling step**
- **Found during:** Task 2 - pkg binary build
- **Issue:** pkg doesn't support ESM modules, threw ERR_REQUIRE_ESM error when running binary
- **Fix:** Added esbuild dependency and bundle.mjs script to convert ESM to CommonJS before pkg compilation
- **Files modified:** apps/backend/package.json, apps/backend/scripts/bundle.mjs (created), apps/backend/scripts/build-binaries.mjs
- **Commit:** Included in cc0e1bd (Task 2 commit)
- **Rationale:** Critical for binary functionality; without bundling, binaries fail to start

**2. [Rule 2 - Missing Critical] Added --no-bytecode flag and updated pkg config**
- **Found during:** Task 2 - initial pkg builds had Babel parse warnings
- **Issue:** pkg bytecode compilation failed with ESM syntax errors
- **Fix:** Added --no-bytecode and --public-packages flags, switched to bundled CommonJS input
- **Files modified:** apps/backend/scripts/build-binaries.mjs
- **Commit:** Included in cc0e1bd (Task 2 commit)
- **Rationale:** Eliminates build warnings, ensures reliable binary generation

## Issues Encountered

### ESM Compatibility with pkg
**Issue:** pkg (packager) doesn't fully support Node.js ESM modules
**Manifestation:** Binary built successfully but threw ERR_REQUIRE_ESM at runtime
**Root cause:** pkg uses CommonJS require() to load entry point, incompatible with ESM
**Resolution:** Two-step build process - esbuild bundles ESM to single CommonJS file, then pkg compiles
**Time impact:** ~3 minutes debugging and implementing bundling solution
**Lessons:** Binary packagers lag behind ESM adoption; bundling step is standard practice for production binaries

### Workspace Dependencies in Binary
**Issue:** pkg struggled to resolve @aios/shared workspace dependency
**Manifestation:** Initial builds included partial assets, runtime errors about missing modules
**Root cause:** pkg doesn't understand pnpm workspace:* protocol
**Resolution:** esbuild bundles all dependencies (including @aios/shared) into single file
**Time impact:** ~2 minutes adjusting pkg config before switching to bundling
**Lessons:** Bundling solves dependency resolution issues for binary packaging

## Next Phase Readiness

**Ready for 03-02 (Frontend API Client)?** ✅ Yes

**Blockers:** None

**Verification:**
- Backend binary runs successfully with dynamic port allocation
- Stdout logging confirmed (BACKEND_PORT:{port} format)
- /health endpoint returns enriched JSON (status, timestamp, uptime)
- Graceful shutdown on SIGTERM/SIGINT tested
- Binary file exists at apps/desktop/src-tauri/binaries/backend-aarch64-apple-darwin

**Next Steps:**
1. Plan 03-02 will create frontend API client to consume /health endpoint
2. Plan 03-03 will configure Tauri sidecar to spawn backend binary
3. Plan 03-04 will implement stdout parsing to extract BACKEND_PORT

**Notes:**
- Binary is 46MB (includes Node.js runtime) - acceptable for desktop app
- esbuild bundle approach enables future optimizations (minification, tree-shaking)
- Current target is macOS ARM only; cross-platform targets deferred until CI/CD setup
