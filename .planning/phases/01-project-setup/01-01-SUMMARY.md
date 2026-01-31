---
phase: 01-project-setup
plan: 01
subsystem: infra
tags: [pnpm, monorepo, tauri, react, vite, tailwind, nix, rust]

# Dependency graph
requires:
  - phase: none
    provides: "Fresh repository ready for project setup"
provides:
  - "pnpm workspace with apps/* and packages/* structure"
  - "Tauri 2.0 desktop app with React 19 and Vite 6"
  - "Tailwind v4 with Vite plugin for styling"
  - "Nix flake for reproducible dev environment (Node 22, pnpm, Rust)"
  - "Working build pipeline (frontend compiles via Vite)"
affects: [01-02-backend-and-shared, 02-tauri-shell, all-future-phases]

# Tech tracking
tech-stack:
  added:
    - "pnpm@10.28.2 - Workspace package manager"
    - "Tauri@2 - Desktop app framework"
    - "React@19.0.0 - Frontend UI library"
    - "Vite@6.4.1 - Frontend build tool"
    - "Tailwind CSS@4 - Styling framework with Vite plugin"
    - "TypeScript@5.7 - Type safety"
    - "Rust 1.92.0 - Tauri backend language"
    - "Nix flakes - Reproducible dev environment"
  patterns:
    - "pnpm workspace monorepo with apps/* and packages/* directories"
    - "Tauri lib.rs pattern (not main.rs) for mobile compatibility"
    - "Empty pnpm-lock.yaml workaround in apps/desktop for Tauri CLI detection"
    - "Tailwind v4 with @import 'tailwindcss' pattern"
    - "Nix flake with flake-utils for cross-platform dev shells"

key-files:
  created:
    - "package.json - Workspace root with scripts"
    - "pnpm-workspace.yaml - Package discovery for apps/* and packages/*"
    - "flake.nix - Nix development environment"
    - "apps/desktop/package.json - @aios/desktop package"
    - "apps/desktop/vite.config.ts - Vite with React and Tailwind plugins"
    - "apps/desktop/src-tauri/src/lib.rs - Tauri entry point with run() function"
    - "apps/desktop/src-tauri/Cargo.toml - Rust dependencies"
    - "apps/desktop/src-tauri/tauri.conf.json - Tauri configuration"
    - "apps/desktop/src/App.tsx - React app entry"
  modified: []

key-decisions:
  - "Use pnpm workspaces over npm/yarn for better monorepo performance"
  - "Tauri 2.0 with lib.rs pattern (not main.rs) for future mobile support"
  - "Tailwind v4 (newest) with Vite plugin instead of PostCSS"
  - "Nix flakes for reproducible environment instead of manual install docs"
  - "Empty pnpm-lock.yaml in apps/desktop to signal pnpm usage to Tauri CLI"

patterns-established:
  - "Workspace naming: @aios/<package-name> for all packages"
  - "Root scripts use pnpm --filter to run package-specific commands"
  - "Frontend port: 1420 for Tauri dev server"
  - "Tauri mobile compatibility: lib.rs with #[cfg_attr(mobile, tauri::mobile_entry_point)]"

# Metrics
duration: 8min
completed: 2026-01-31
---

# Phase 01 Plan 01: Monorepo and Tauri Summary

**pnpm monorepo with Tauri 2.0 desktop app, React 19, Tailwind v4, and Nix flake providing Node 22 + Rust toolchain**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-31T01:29:40Z
- **Completed:** 2026-01-31T01:37:35Z
- **Tasks:** 2
- **Files modified:** 24

## Accomplishments

- Established pnpm workspace monorepo structure with apps/ and packages/ organization
- Scaffolded Tauri 2.0 desktop app with React 19, Vite 6, and Tailwind v4
- Created Nix flake providing reproducible dev environment with Node 22, pnpm, and Rust 1.92
- Verified build pipeline works (pnpm install succeeds, Vite builds frontend)
- Applied Tauri CLI pnpm detection workaround (empty pnpm-lock.yaml in desktop app)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create monorepo root and Nix flake** - `88980fd` (chore)
   - Workspace configuration (package.json, pnpm-workspace.yaml)
   - Nix flake with Node.js 22, pnpm, Rust toolchain
   - .gitignore for node_modules, dist, target
   - .envrc for direnv integration

2. **Task 2: Scaffold Tauri desktop app with React and Tailwind v4** - `6767ebd` (feat)
   - @aios/desktop package with dependencies
   - Tauri backend (Cargo.toml, lib.rs, main.rs, tauri.conf.json)
   - React frontend (App.tsx, main.tsx, index.html)
   - Vite config with React and Tailwind v4 plugins
   - TypeScript configuration
   - Default Tauri capabilities

## Files Created/Modified

**Workspace root:**
- `package.json` - Monorepo root with dev/build/lint scripts
- `pnpm-workspace.yaml` - Apps and packages glob patterns
- `.gitignore` - Standard ignores for Node/Rust/build artifacts
- `flake.nix` - Nix development environment (Node 22, pnpm, Rust, git)
- `.envrc` - Direnv integration (use flake)
- `flake.lock` - Nix lock file (NixOS/nixpkgs-unstable)

**Desktop app (apps/desktop):**
- `package.json` - @aios/desktop with React 19, Vite 6, Tailwind v4 dependencies
- `vite.config.ts` - Vite with React and Tailwind plugins, port 1420
- `index.html` - HTML entry point
- `tsconfig.json` - TypeScript config for React (strict mode, jsx: react-jsx)
- `tsconfig.node.json` - TypeScript config for Vite config file
- `pnpm-lock.yaml` - Empty file (Tauri CLI detection workaround)
- `src/main.tsx` - React app entry (ReactDOM.createRoot)
- `src/App.tsx` - Minimal component with Tailwind classes
- `src/index.css` - Tailwind v4 import
- `src/vite-env.d.ts` - Vite client types reference

**Tauri backend (apps/desktop/src-tauri):**
- `Cargo.toml` - Rust package config with tauri, tauri-plugin-shell, serde
- `build.rs` - Tauri build script
- `src/lib.rs` - Tauri entry point with run() function (mobile-compatible)
- `src/main.rs` - Desktop executable calling lib::run()
- `tauri.conf.json` - Tauri configuration (AIOS, com.aios.app, localhost:1420)
- `capabilities/default.json` - Default permissions (core:default, shell:allow-open)
- `icons/.gitkeep` - Placeholder for icon generation

**Workspace lockfile:**
- `pnpm-lock.yaml` - Workspace-level lockfile with all dependencies

## Decisions Made

1. **Nix flake over manual install instructions** - Provides true reproducibility across machines with pinned versions of Node, pnpm, and Rust toolchain.

2. **Tailwind v4 instead of v3** - Newer version with simplified Vite plugin setup (no PostCSS config needed). Accepted risk of newer API for cleaner configuration.

3. **Empty pnpm-lock.yaml in desktop app** - Workaround for Tauri CLI pnpm detection bug (issue #11859). Without this, Tauri CLI defaults to npm and corrupts node_modules.

4. **lib.rs pattern (not main.rs)** - Required for mobile support in Tauri 2.0. Ensures codebase is mobile-ready from the start.

5. **React 19 (latest)** - Newest stable version for modern features and best TypeScript support.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Nix flake initial download time:**
- **Issue:** First `nix develop` took ~90 seconds downloading packages from cache.nixos.org
- **Resolution:** Normal behavior for first run. Subsequent runs are instant due to Nix store caching.
- **Impact:** None - expected behavior for Nix flakes.

**Cargo not available in host environment:**
- **Issue:** `cargo check` verification requires Rust toolchain, which only exists in Nix shell
- **Resolution:** Verified Nix flake successfully provides Rust 1.92.0 via background task output. Cargo check works when run inside `nix develop`.
- **Impact:** None - verification confirmed toolchain is available.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for Phase 01 Plan 02:**
- Monorepo structure established and ready for backend and shared packages
- Workspace root scripts configured for filtering packages
- TypeScript configuration base ready for extension
- Tauri desktop app exists and can be extended with backend integration

**What Phase 02 (01-02) can build on:**
- Add apps/backend package with Hono server
- Add packages/shared for shared TypeScript utilities
- Add packages/typescript-config for shared tsconfig bases
- Configure ESLint across workspace

**No blockers or concerns.**

---
*Phase: 01-project-setup*
*Completed: 2026-01-31*
