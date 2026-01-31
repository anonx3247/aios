# Phase 3: Node Backend - Research

**Researched:** 2026-01-31
**Domain:** Tauri 2.0 with Node.js sidecar (Hono), React + Vite frontend, HTTP communication
**Confidence:** HIGH

## Summary

This research covers the integration of a Node.js backend (Hono HTTP server) as a Tauri sidecar, a React frontend with Vite and Tailwind CSS v4, and HTTP-based communication between frontend and backend. The standard approach uses Tauri 2.0's shell plugin for sidecar management, pkg to compile Node.js into platform-specific binaries, Hono for the HTTP server with dynamic port allocation, React with Vite for the frontend, and Tauri's HTTP plugin or standard fetch for frontend-backend communication.

Tauri 2.0 provides robust sidecar support for bundling external binaries with the app, eliminating the need for users to install Node.js. The shell plugin handles process spawning with `kill_on_drop(true)` for automatic cleanup, though developers must manually implement port discovery via stdout parsing. Hono offers a minimal HTTP server with dynamic port configuration through environment variables or the `serve()` callback. React with Vite provides fast development and production builds, while Tailwind v4's Vite plugin offers zero-config integration.

Key challenges include: (1) compiling Node.js to platform-specific binaries with the correct target triple naming convention, (2) communicating the dynamically allocated port from the sidecar to the frontend, (3) ensuring proper process cleanup to prevent orphan processes, and (4) managing CORS/CSP for localhost HTTP communication in the Tauri WebView.

**Primary recommendation:** Use pkg to compile the Hono server to binaries, configure the sidecar in tauri.conf.json with proper target triple suffixes, spawn the sidecar from Rust on app startup, parse stdout to extract the dynamic port, store it in Tauri state, expose it to the frontend via a command, use Tauri's HTTP plugin for frontend requests to avoid CORS issues, and rely on `kill_on_drop(true)` for process cleanup.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @tauri-apps/plugin-shell | 2.3+ | Sidecar process management with kill_on_drop | Official Tauri plugin for spawning external binaries |
| pkg (@yao-pkg/pkg) | Latest | Compile Node.js to standalone executables | Community-maintained fork of Vercel pkg, supports Node 18+ |
| hono | 4.x | Lightweight HTTP server framework | Modern, fast, minimal Node.js server with Web Standard APIs |
| @hono/node-server | 1.x | Node.js adapter for Hono | Official adapter for running Hono on Node.js runtime |
| react | 19.x | UI library | Already in project, standard for Tauri frontends |
| vite | 6.x | Frontend build tool | Already in project, official Tauri frontend integration |
| @tailwindcss/vite | 4.x | Tailwind CSS v4 Vite plugin | Official Tailwind v4 plugin with zero-config setup |
| @tauri-apps/plugin-http | 2.x | HTTP client for frontend | Official plugin, bypasses CORS for localhost requests |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/api | 2.x | Tauri frontend bindings | For invoke, window events, IPC communication |
| tsx | 4.x | TypeScript execution for Node backend dev | Development-only, for `pnpm dev` in backend |
| get-port | Latest | Find available port dynamically | Alternative to port 0 for more control |
| @vitejs/plugin-react | 4.x | React support in Vite | Required for JSX/TSX compilation |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| pkg | Bundling Node.js runtime + JS files | pkg creates smaller, self-contained binaries; bundling Node runtime requires users to have compatible Node version |
| Hono | Express, Fastify, Koa | Hono is newer, lighter, uses Web Standards; Express is more mature but heavier |
| Tauri HTTP plugin | Standard fetch | Tauri HTTP plugin avoids CORS issues; standard fetch works but requires CSP configuration |
| Port 0 dynamic allocation | get-port library | Port 0 is simpler, OS-managed; get-port offers more control and guarantees |
| Stdout parsing | IPC via Tauri commands | Stdout is universal, works with any sidecar; IPC requires sidecar to use Tauri bindings |

**Installation:**
```bash
# Backend dependencies (already in apps/backend/package.json)
pnpm --filter @aios/backend add hono @hono/node-server
pnpm --filter @aios/backend add -D tsx @yao-pkg/pkg @types/node

# Frontend dependencies (already in apps/desktop/package.json)
pnpm --filter @aios/desktop add react react-dom @tauri-apps/api
pnpm --filter @aios/desktop add -D @vitejs/plugin-react vite @tailwindcss/vite tailwindcss

# Tauri plugins (Rust side)
cd apps/desktop/src-tauri
cargo add tauri-plugin-shell
cargo add tauri-plugin-http

# Initialize plugins in main.rs
pnpm run tauri add shell
pnpm run tauri add http
```

## Architecture Patterns

### Recommended Project Structure
```
apps/
├── backend/
│   ├── src/
│   │   ├── index.ts              # Hono server entry, port allocation
│   │   ├── routes/               # API route handlers
│   │   │   ├── health.ts         # Health check endpoint
│   │   │   └── agents.ts         # Agent orchestration endpoints
│   │   └── lib/                  # Shared backend utilities
│   ├── dist/                     # TypeScript build output
│   ├── package.json              # pkg build script, dependencies
│   └── tsconfig.json
├── desktop/
│   ├── src/
│   │   ├── App.tsx               # React root component
│   │   ├── main.tsx              # React entry point
│   │   ├── index.css             # Tailwind imports
│   │   └── lib/
│   │       ├── api.ts            # HTTP client wrapper for backend
│   │       └── tauri.ts          # Tauri command wrappers
│   ├── src-tauri/
│   │   ├── src/
│   │   │   ├── lib.rs            # Sidecar spawn, port discovery
│   │   │   ├── commands/
│   │   │   │   └── backend.rs    # get_backend_port command
│   │   │   └── state.rs          # Store backend port in state
│   │   ├── binaries/
│   │   │   ├── backend-x86_64-apple-darwin        # macOS Intel
│   │   │   ├── backend-aarch64-apple-darwin       # macOS ARM
│   │   │   ├── backend-x86_64-unknown-linux-gnu   # Linux x64
│   │   │   └── backend-x86_64-pc-windows-msvc.exe # Windows
│   │   ├── capabilities/
│   │   │   └── default.json      # Shell and HTTP permissions
│   │   └── tauri.conf.json       # Sidecar externalBin config
│   ├── dist/                     # Vite build output
│   ├── vite.config.ts            # React + Tailwind plugins
│   └── package.json
```

### Pattern 1: Compiling Node.js Backend with pkg
**What:** Use pkg to compile TypeScript/JavaScript to platform-specific binaries with target triple suffixes required by Tauri.
**When to use:** Building for production or testing sidecar integration.
**Example:**
```json
// Source: https://github.com/yao-pkg/pkg
// apps/backend/package.json
{
  "scripts": {
    "build": "tsc && node scripts/build-binaries.js"
  },
  "pkg": {
    "assets": ["dist/**/*"],
    "targets": [
      "node18-macos-x64",
      "node18-macos-arm64",
      "node18-linux-x64",
      "node18-win-x64"
    ],
    "outputPath": "../desktop/src-tauri/binaries"
  }
}
```

**Build script to rename with target triples:**
```javascript
// Source: https://v2.tauri.app/learn/sidecar-nodejs/
// apps/backend/scripts/build-binaries.js
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

const targets = [
  { pkg: 'node18-macos-x64', suffix: 'x86_64-apple-darwin' },
  { pkg: 'node18-macos-arm64', suffix: 'aarch64-apple-darwin' },
  { pkg: 'node18-linux-x64', suffix: 'x86_64-unknown-linux-gnu' },
  { pkg: 'node18-win-x64', suffix: 'x86_64-pc-windows-msvc.exe' },
];

const outputDir = '../desktop/src-tauri/binaries';
fs.mkdirSync(outputDir, { recursive: true });

targets.forEach(({ pkg, suffix }) => {
  console.log(`Building for ${pkg}...`);
  execSync(`pkg dist/index.js --target ${pkg} --output ${outputDir}/backend-temp`, {
    stdio: 'inherit'
  });

  // Rename to Tauri's expected format
  const tempFile = suffix.endsWith('.exe')
    ? `${outputDir}/backend-temp.exe`
    : `${outputDir}/backend-temp`;
  const finalFile = `${outputDir}/backend-${suffix}`;

  fs.renameSync(tempFile, finalFile);
  fs.chmodSync(finalFile, 0o755); // Make executable
  console.log(`Created ${finalFile}`);
});
```

### Pattern 2: Hono Server with Dynamic Port Allocation
**What:** Configure Hono server to use dynamic port via environment variable or port 0, log the assigned port to stdout for Tauri to capture.
**When to use:** All sidecar HTTP server scenarios where port conflicts must be avoided.
**Example:**
```typescript
// Source: https://hono.dev/docs/getting-started/nodejs
// apps/backend/src/index.ts
import { serve } from '@hono/node-server';
import { Hono } from 'hono';

const app = new Hono();

app.get('/health', (c) => c.json({ status: 'ok', timestamp: Date.now() }));

// Dynamic port: PORT env var, or 0 for OS assignment
const port = Number(process.env.PORT) || 0;

const server = serve({
  fetch: app.fetch,
  port
}, (info) => {
  // CRITICAL: Log to stdout in parseable format for Tauri
  console.log(`BACKEND_PORT:${info.port}`);
  console.error(`Backend server started on http://localhost:${info.port}`);
});

// Graceful shutdown
process.on('SIGINT', () => {
  console.error('Shutting down backend server...');
  server.close(() => process.exit(0));
});

process.on('SIGTERM', () => {
  console.error('Received SIGTERM, shutting down...');
  server.close(() => process.exit(0));
});
```

### Pattern 3: Tauri Sidecar Configuration and Spawning
**What:** Configure sidecar in tauri.conf.json, spawn in lib.rs setup, parse stdout to extract port, store in state.
**When to use:** All long-running sidecar processes that need communication with frontend.
**Example:**
```json
// Source: https://v2.tauri.app/develop/sidecar/
// apps/desktop/src-tauri/tauri.conf.json
{
  "bundle": {
    "externalBin": [
      "binaries/backend"
    ]
  },
  "plugins": {
    "shell": {
      "open": true
    }
  }
}
```

```json
// Source: https://v2.tauri.app/security/capabilities/
// apps/desktop/src-tauri/capabilities/default.json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    {
      "identifier": "shell:allow-execute",
      "allow": [
        {
          "name": "binaries/backend",
          "sidecar": true,
          "args": true
        }
      ]
    },
    "http:default",
    {
      "identifier": "http:allow-fetch",
      "allow": [
        {
          "url": "http://localhost:*"
        }
      ]
    }
  ]
}
```

```rust
// Source: https://v2.tauri.app/reference/javascript/shell/
// apps/desktop/src-tauri/src/lib.rs
use tauri::{Manager, State};
use tauri_plugin_shell::ShellExt;
use std::sync::{Arc, Mutex};
use tauri_plugin_shell::process::CommandEvent;

pub struct AppState {
    pub backend_port: Arc<Mutex<Option<u16>>>,
}

#[tauri::command]
fn get_backend_port(state: State<'_, AppState>) -> Result<u16, String> {
    state.backend_port.lock().unwrap()
        .ok_or_else(|| "Backend port not available".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            let state = AppState {
                backend_port: Arc::new(Mutex::new(None)),
            };

            // Spawn backend sidecar
            let shell = app.shell();
            let sidecar_command = shell.sidecar("backend")?;
            let (mut rx, mut _child) = sidecar_command.spawn()?;

            let port_state = state.backend_port.clone();

            // Listen for stdout to extract port
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stdout(line) => {
                            let line_str = String::from_utf8_lossy(&line);
                            if let Some(port_str) = line_str.strip_prefix("BACKEND_PORT:") {
                                if let Ok(port) = port_str.trim().parse::<u16>() {
                                    *port_state.lock().unwrap() = Some(port);
                                    println!("Backend port discovered: {}", port);
                                }
                            }
                        }
                        CommandEvent::Stderr(line) => {
                            eprintln!("Backend stderr: {}", String::from_utf8_lossy(&line));
                        }
                        CommandEvent::Error(err) => {
                            eprintln!("Backend error: {}", err);
                        }
                        CommandEvent::Terminated(payload) => {
                            eprintln!("Backend terminated with code {:?}", payload.code);
                        }
                        _ => {}
                    }
                }
            });

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_backend_port])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Pattern 4: React Frontend with HTTP Communication to Backend
**What:** Use Tauri's invoke to get backend port, use Tauri HTTP plugin (or fetch) to call backend API.
**When to use:** All frontend-backend HTTP communication scenarios.
**Example:**
```typescript
// Source: https://v2.tauri.app/reference/javascript/http/
// apps/desktop/src/lib/api.ts
import { invoke } from '@tauri-apps/api/core';
import { fetch } from '@tauri-apps/plugin-http';

let cachedPort: number | null = null;

async function getBackendPort(): Promise<number> {
  if (cachedPort !== null) return cachedPort;
  cachedPort = await invoke<number>('get_backend_port');
  return cachedPort;
}

export async function callBackend<T>(endpoint: string, init?: RequestInit): Promise<T> {
  const port = await getBackendPort();
  const url = `http://localhost:${port}${endpoint}`;
  const response = await fetch(url, init);

  if (!response.ok) {
    throw new Error(`Backend request failed: ${response.statusText}`);
  }

  return response.json();
}

export async function healthCheck(): Promise<{ status: string; timestamp: number }> {
  return callBackend('/health');
}
```

```tsx
// Source: https://www.willhart.io/post/tauri-create-react-app-tutorial-part3/
// apps/desktop/src/App.tsx
import { useEffect, useState } from 'react';
import { healthCheck } from './lib/api';

function App() {
  const [backendStatus, setBackendStatus] = useState<string>('connecting...');

  useEffect(() => {
    async function checkBackend() {
      try {
        const health = await healthCheck();
        setBackendStatus(`connected (${health.status})`);
      } catch (error) {
        setBackendStatus(`error: ${error}`);
      }
    }

    checkBackend();
    const interval = setInterval(checkBackend, 10000); // Poll every 10s

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-zinc-950 text-white flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-6xl font-bold mb-4">AIOS</h1>
        <p className="text-zinc-400 text-lg">Agent launcher and task manager</p>
        <p className="text-sm text-zinc-600 mt-4">Backend: {backendStatus}</p>
      </div>
    </div>
  );
}

export default App;
```

### Pattern 5: Vite Configuration for React and Tailwind v4
**What:** Configure Vite with React and Tailwind v4 plugins, set proper build target for Tauri WebView.
**When to use:** All Tauri + React + Vite + Tailwind projects.
**Example:**
```typescript
// Source: https://v2.tauri.app/start/frontend/vite/
// Source: https://tailwindcss.com/blog/tailwindcss-v4
// apps/desktop/vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [
    react(),
    tailwindcss()
  ],

  // Tauri expects frontend to be served from localhost:1420 in dev
  server: {
    port: 1420,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || false,
  },

  // Tauri WebView compatibility
  build: {
    target: process.env.TAURI_PLATFORM === 'windows'
      ? 'chrome105'
      : 'safari13',
    minify: !process.env.TAURI_DEBUG,
    sourcemap: !!process.env.TAURI_DEBUG,
  },

  envPrefix: ['VITE_', 'TAURI_ENV_*'],

  clearScreen: false,
});
```

```css
/* Source: https://tailwindcss.com/blog/tailwindcss-v4 */
/* apps/desktop/src/index.css */
@import "tailwindcss";

/* Custom styles here */
```

```json
// apps/desktop/src-tauri/tauri.conf.json (build section)
{
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist"
  }
}
```

### Anti-Patterns to Avoid
- **Not using kill_on_drop:** Forgetting `.kill_on_drop(true)` leads to orphan backend processes when Tauri app exits.
- **Hardcoded ports:** Using fixed ports (3000, 8080) causes conflicts when multiple instances run or port is occupied.
- **Ignoring sidecar stderr:** Backend errors are only visible on stderr; not monitoring it makes debugging impossible.
- **Synchronous port waiting:** Blocking until port is discovered freezes app startup; use async spawn and state update.
- **Missing target triple suffixes:** Tauri requires exact binary names like `backend-x86_64-apple-darwin`; wrong names cause "binary not found" errors.
- **Using standard fetch without CORS config:** Standard fetch to localhost requires CSP configuration; Tauri HTTP plugin bypasses this.
- **Not handling backend startup failure:** Backend may fail to start (port in use, missing deps); frontend should handle missing port gracefully.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Node.js binary compilation | Manual bundling of Node.js runtime + source | pkg (@yao-pkg/pkg) | Handles Node.js embedding, platform-specific builds, asset bundling automatically |
| Dynamic port allocation | Custom port scanning loop | Port 0 or get-port library | OS-managed port 0 is simple and reliable; get-port adds retry logic and guarantees |
| Process stdout parsing | Custom regex or string splitting | Line-based parsing with prefix format | Prefix format (`KEY:value`) is robust, handles buffering, supports multiple keys |
| HTTP client CORS bypass | Custom CSP configuration | @tauri-apps/plugin-http | Plugin routes requests through Rust backend, avoiding WebView CORS entirely |
| Sidecar process cleanup | Manual process tracking and kill | Shell plugin's kill_on_drop(true) | Automatic cleanup on app exit, handles edge cases, prevents zombie/orphan processes |
| React + Vite + Tailwind setup | Manual PostCSS config, content globs | Tailwind v4 @tailwindcss/vite plugin | Zero-config, auto-discovers template files, optimizes for production automatically |

**Key insight:** Tauri 2.0's sidecar system, pkg tooling, and modern web frameworks provide battle-tested solutions for all common patterns in desktop apps with backend services. Custom solutions introduce platform-specific bugs, security risks, and maintenance burden.

## Common Pitfalls

### Pitfall 1: Sidecar Binary Not Found Errors
**What goes wrong:** Tauri fails to spawn sidecar with "binary not found" or "permission denied" errors.
**Why it happens:** Binary naming doesn't match Tauri's expected format (`name-$TARGET_TRIPLE`), or executable bit not set on Unix.
**How to avoid:** Use exact target triple from `rustc --print host-tuple`. After building, verify files exist with correct names and run `chmod +x` on Unix binaries. Test on each target platform.
**Warning signs:** "Failed to spawn sidecar" errors, different behavior between platforms.

### Pitfall 2: Orphan Backend Processes
**What goes wrong:** Backend continues running after Tauri app exits, consuming resources and ports.
**Why it happens:** Forgetting `.kill_on_drop(true)` in Command builder, or dropping Child handle without cleanup.
**How to avoid:** Always set `.kill_on_drop(true)` when spawning sidecar. Store Child handle in state if manual control needed, ensure cleanup on app exit.
**Warning signs:** Multiple backend processes in task manager, port already in use on restart.

### Pitfall 3: Port Discovery Race Condition
**What goes wrong:** Frontend tries to connect to backend before port is discovered, requests fail with connection refused.
**Why it happens:** Async port discovery takes time; frontend makes requests immediately on mount.
**How to avoid:** Expose backend readiness state to frontend. Use polling or event emission when port is available. Add retry logic to API client.
**Warning signs:** Intermittent "Backend port not available" errors, works on retry but not first load.

### Pitfall 4: CORS Errors with Standard Fetch
**What goes wrong:** Frontend fetch to localhost:PORT fails with CORS policy errors despite same machine.
**Why it happens:** Tauri WebView treats custom protocol (`tauri://localhost`) and `http://localhost` as different origins.
**How to avoid:** Use @tauri-apps/plugin-http instead of standard fetch, or configure CSP to allow localhost. HTTP plugin is recommended approach.
**Warning signs:** Console errors mentioning "CORS policy", "blocked by CORS", works in browser but not Tauri.

### Pitfall 5: pkg Compilation Failures with Native Modules
**What goes wrong:** pkg fails to bundle backend or binary crashes on startup with "module not found" errors for native dependencies.
**Why it happens:** pkg cannot bundle native (.node) modules; they must be distributed separately or avoided.
**How to avoid:** Avoid native dependencies in sidecar code. If unavoidable, use pkg's assets feature to include .node files and load them at runtime. Consider pure JavaScript alternatives.
**Warning signs:** "Cannot find module" errors for native packages, pkg build warnings about native modules.

### Pitfall 6: Stdout Buffering Delays
**What goes wrong:** Port log appears in backend console but Tauri never receives it, causing timeout waiting for port.
**Why it happens:** Node.js buffers stdout; `console.log` may not flush immediately, especially in packaged binaries.
**How to avoid:** Use `console.error` (unbuffered) or call `process.stdout.write()` with explicit flush. Add newline to ensure line buffering works.
**Warning signs:** Port discovery timeout, logs appear late or out of order, works in dev but not in production binary.

### Pitfall 7: Multiple Backend Instances on Hot Reload
**What goes wrong:** During development, Vite hot reload or Tauri rebuild spawns new backend instances without killing old ones.
**Why it happens:** kill_on_drop only works on app exit, not on rebuild. Development restarts don't always trigger cleanup.
**How to avoid:** In development, use a separate `pnpm dev:backend` process instead of sidecar. Only test sidecar in production builds. Add process detection to prevent multiple starts.
**Warning signs:** Port conflicts, multiple "Backend server started" logs, resource exhaustion during development.

### Pitfall 8: Windows Path Escaping in Sidecar Args
**What goes wrong:** Sidecar spawns but fails with "file not found" when passing paths as arguments on Windows.
**Why it happens:** Windows path backslashes need escaping in JSON and Rust strings; single backslash causes parsing errors.
**How to avoid:** Use forward slashes in paths (Node.js normalizes them on Windows) or double backslashes. Test argument passing on Windows explicitly.
**Warning signs:** Works on macOS/Linux but fails on Windows, path-related errors in backend logs.

### Pitfall 9: Tailwind v4 Migration Issues
**What goes wrong:** Tailwind v3 config files (tailwind.config.js) conflict with v4 plugin, causing build failures or missing styles.
**Why it happens:** Tailwind v4 changed configuration model from JS config to CSS-first with Vite plugin. Old config files override new behavior.
**How to avoid:** Remove tailwind.config.js and postcss.config.js when using v4 plugin. Move customizations to CSS using `@theme` directive. Follow v4 migration guide.
**Warning signs:** Styles not applied, build errors mentioning "invalid configuration", v3 syntax not working.

### Pitfall 10: TypeScript Path Aliases Not Resolved in Vite
**What goes wrong:** TypeScript path aliases (e.g., `@/components`) work in IDE but cause "module not found" errors in Vite build.
**Why it happens:** Vite doesn't read tsconfig.json paths by default; requires explicit alias configuration.
**How to avoid:** Use vite-tsconfig-paths plugin or manually configure `resolve.alias` in vite.config.ts to match tsconfig paths. Prefer workspace protocol for cross-package imports.
**Warning signs:** IDE auto-import works but build fails, different behavior between dev and build.

## Code Examples

Verified patterns from official sources:

### Complete Backend Server with Graceful Shutdown
```typescript
// Source: https://hono.dev/docs/getting-started/nodejs
import { serve } from '@hono/node-server';
import { Hono } from 'hono';
import type { Server } from 'http';

const app = new Hono();

// Health check endpoint
app.get('/health', (c) => c.json({
  status: 'ok',
  timestamp: Date.now(),
  uptime: process.uptime()
}));

// Example API endpoint
app.post('/api/agents/run', async (c) => {
  const body = await c.req.json();
  // Agent orchestration logic here
  return c.json({ success: true, runId: 'mock-id' });
});

const port = Number(process.env.PORT) || 0;

let server: Server;

server = serve({ fetch: app.fetch, port }, (info) => {
  // Log to stdout for Tauri to parse
  console.log(`BACKEND_PORT:${info.port}`);
  // Log to stderr for debugging (doesn't interfere with stdout parsing)
  console.error(`[Backend] Server ready on http://localhost:${info.port}`);
});

// Graceful shutdown handlers
const shutdown = () => {
  console.error('[Backend] Shutting down...');
  server.close(() => {
    console.error('[Backend] Server closed');
    process.exit(0);
  });
};

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

// Export app type for frontend type safety (optional)
export type AppType = typeof app;
```

### Frontend API Client with Retry Logic
```typescript
// Source: https://v2.tauri.app/reference/javascript/http/
import { invoke } from '@tauri-apps/api/core';
import { fetch } from '@tauri-apps/plugin-http';

class BackendClient {
  private port: number | null = null;
  private portPromise: Promise<number> | null = null;

  async getPort(): Promise<number> {
    if (this.port !== null) return this.port;

    // Avoid race condition with multiple simultaneous calls
    if (this.portPromise) return this.portPromise;

    this.portPromise = invoke<number>('get_backend_port');
    this.port = await this.portPromise;
    this.portPromise = null;

    return this.port;
  }

  async call<T>(endpoint: string, init?: RequestInit): Promise<T> {
    const port = await this.getPort();
    const url = `http://localhost:${port}${endpoint}`;

    const response = await fetch(url, {
      ...init,
      headers: {
        'Content-Type': 'application/json',
        ...init?.headers,
      },
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Backend error (${response.status}): ${error}`);
    }

    return response.json();
  }

  async healthCheck() {
    return this.call<{ status: string; timestamp: number }>('/health');
  }
}

export const backend = new BackendClient();
```

### React Component with Backend Connection Status
```tsx
import { useEffect, useState } from 'react';
import { backend } from './lib/api';

function App() {
  const [status, setStatus] = useState<'connecting' | 'connected' | 'error'>('connecting');
  const [error, setError] = useState<string>('');

  useEffect(() => {
    let mounted = true;
    let retryCount = 0;
    const maxRetries = 10;

    async function connect() {
      while (mounted && retryCount < maxRetries) {
        try {
          await backend.healthCheck();
          if (mounted) {
            setStatus('connected');
            setError('');
          }
          return;
        } catch (err) {
          retryCount++;
          console.error(`Backend connection failed (attempt ${retryCount}):`, err);

          if (retryCount < maxRetries) {
            await new Promise(resolve => setTimeout(resolve, 1000 * retryCount));
          } else {
            if (mounted) {
              setStatus('error');
              setError(`Failed after ${maxRetries} attempts`);
            }
          }
        }
      }
    }

    connect();

    return () => {
      mounted = false;
    };
  }, []);

  return (
    <div className="min-h-screen bg-zinc-950 text-white flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-6xl font-bold mb-4">AIOS</h1>
        <p className="text-zinc-400 text-lg">Agent launcher and task manager</p>
        <div className="mt-4 text-sm">
          {status === 'connecting' && (
            <p className="text-yellow-500">Connecting to backend...</p>
          )}
          {status === 'connected' && (
            <p className="text-green-500">Backend connected</p>
          )}
          {status === 'error' && (
            <p className="text-red-500">Backend error: {error}</p>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
```

### Sidecar Spawn with Comprehensive Error Handling
```rust
// Source: https://v2.tauri.app/reference/javascript/shell/
use tauri::{Manager, State, AppHandle};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AppState {
    pub backend_port: Arc<Mutex<Option<u16>>>,
    pub backend_ready: Arc<Mutex<bool>>,
}

pub fn spawn_backend_sidecar(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let shell = app.shell();

    let sidecar_command = shell
        .sidecar("backend")
        .map_err(|e| format!("Failed to create sidecar command: {}", e))?;

    let (mut rx, mut child) = sidecar_command
        .spawn()
        .map_err(|e| format!("Failed to spawn sidecar: {}", e))?;

    let port_state = state.backend_port.clone();
    let ready_state = state.backend_ready.clone();

    tauri::async_runtime::spawn(async move {
        let mut startup_timeout = tokio::time::interval(Duration::from_secs(30));
        startup_timeout.tick().await; // First tick completes immediately

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    match event {
                        CommandEvent::Stdout(line) => {
                            let line_str = String::from_utf8_lossy(&line);

                            // Parse BACKEND_PORT:12345 format
                            if let Some(port_str) = line_str.strip_prefix("BACKEND_PORT:") {
                                if let Ok(port) = port_str.trim().parse::<u16>() {
                                    *port_state.lock().unwrap() = Some(port);
                                    *ready_state.lock().unwrap() = true;
                                    println!("[Tauri] Backend ready on port {}", port);
                                }
                            }
                        }
                        CommandEvent::Stderr(line) => {
                            eprintln!("[Backend] {}", String::from_utf8_lossy(&line));
                        }
                        CommandEvent::Error(err) => {
                            eprintln!("[Backend] Error: {}", err);
                            *ready_state.lock().unwrap() = false;
                        }
                        CommandEvent::Terminated(payload) => {
                            eprintln!("[Backend] Terminated with code {:?}", payload.code);
                            *ready_state.lock().unwrap() = false;
                            break;
                        }
                        _ => {}
                    }
                }
                _ = startup_timeout.tick() => {
                    if !*ready_state.lock().unwrap() {
                        eprintln!("[Backend] Startup timeout - backend did not report port");
                    }
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn get_backend_port(state: State<'_, AppState>) -> Result<u16, String> {
    let port = state.backend_port.lock().unwrap();
    port.ok_or_else(|| "Backend not ready".to_string())
}

#[tauri::command]
fn backend_ready(state: State<'_, AppState>) -> bool {
    *state.backend_ready.lock().unwrap()
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Vercel's pkg | @yao-pkg/pkg fork | 2023 | Community maintains pkg with Node 18+ support after Vercel archived project |
| Tailwind v3 JS config | Tailwind v4 CSS-first + Vite plugin | Dec 2024 | Zero-config setup, faster builds, native CSS layer support |
| Express for simple APIs | Hono with Web Standards | 2023-2024 | Lighter, faster, modern fetch-like APIs, better TypeScript support |
| Manual CORS configuration | Tauri HTTP plugin routing through Rust | Tauri 2.0 (2024) | Bypass CORS entirely, consistent with security model |
| Tauri 1.x shell API | Tauri 2.0 shell plugin with permissions | Oct 2024 | Granular permissions, better security, similar API |
| PostCSS + autoprefixer manual setup | Vite built-in CSS processing | Vite 5+ | Automatic vendor prefixing, Lightning CSS in production |

**Deprecated/outdated:**
- **Vercel pkg (official)**: Archived by Vercel; use @yao-pkg/pkg maintained fork instead
- **Tailwind v3 config files**: Don't use tailwind.config.js with v4; configuration via CSS `@theme` directive
- **content globs in PostCSS**: Tailwind v4 auto-discovers files; no manual configuration needed
- **tauri.conf.json allowlist**: Replaced by capabilities system in Tauri 2.0
- **Standard fetch without plugin**: Works but requires CSP changes; HTTP plugin is recommended approach

## Open Questions

Things that couldn't be fully resolved:

1. **Backend restart strategy**
   - What we know: Sidecar terminates when app exits via kill_on_drop; CommandEvent::Terminated fires on crash
   - What's unclear: Whether to implement automatic restart on crash, or let app fail fast
   - Recommendation: Start without auto-restart (fail fast). Add restart logic in later phase if needed. Log termination events for debugging.

2. **Port allocation strategy**
   - What we know: Port 0 gives OS-assigned port; get-port library provides more control
   - What's unclear: Whether port conflicts are likely enough to warrant get-port vs simpler port 0
   - Recommendation: Start with port 0 (simpler). Switch to get-port if issues arise in testing. Document port range if needed.

3. **Backend binary size optimization**
   - What we know: pkg bundles entire Node.js runtime; binaries are 40-60MB per platform
   - What's unclear: Whether size matters for desktop app distribution, or if compression/delta updates mitigate it
   - Recommendation: Accept initial size. Investigate pkg's --compress option if distribution size becomes issue. Consider splitting large dependencies.

4. **Development workflow for sidecar testing**
   - What we know: Sidecar only works in production builds; development can't test sidecar behavior easily
   - What's unclear: Best workflow to test sidecar integration without full builds every time
   - Recommendation: Run backend separately (`pnpm dev:backend`) during development. Hardcode port 3001 in dev mode. Test sidecar integration only in production builds before releases.

5. **HTTP vs WebSocket for backend communication**
   - What we know: Phase 3 requirements specify HTTP; long-polling or SSE needed for push notifications
   - What's unclear: Whether HTTP is sufficient for real-time agent status updates, or if WebSocket will be needed later
   - Recommendation: Start with HTTP + polling (simpler, meets requirements). Add WebSocket in later phase if real-time requirements emerge. Hono supports WebSocket if needed.

## Sources

### Primary (HIGH confidence)
- [Tauri 2.0 Sidecar Documentation](https://v2.tauri.app/develop/sidecar/) - External binary bundling, configuration
- [Tauri Shell Plugin](https://v2.tauri.app/plugin/shell/) - Command API, process spawning, kill_on_drop
- [Node.js as Sidecar Guide](https://v2.tauri.app/learn/sidecar-nodejs/) - pkg compilation, target triples, permissions
- [Tauri HTTP Plugin](https://v2.tauri.app/reference/javascript/http/) - fetch API, CORS bypass, localhost requests
- [Hono Documentation - Node.js](https://hono.dev/docs/getting-started/nodejs) - Server setup, port configuration, serve options
- [Tailwind CSS v4 Release](https://tailwindcss.com/blog/tailwindcss-v4) - Vite plugin, CSS-first config, migration guide
- [Vite Tauri Integration](https://v2.tauri.app/start/frontend/vite/) - Configuration, build targets, dev server setup
- [@yao-pkg/pkg on npm](https://www.npmjs.com/package/@yao-pkg/pkg) - Community-maintained pkg fork with Node 18+ support
- [Tauri Inter-Process Communication](https://v2.tauri.app/concept/inter-process-communication/) - Commands, events, serialization

### Secondary (MEDIUM confidence)
- [Adding Node.js server to Tauri App as a sidecar - DEV Community](https://dev.to/zaid_sunasra/adding-nodejs-server-to-tauri-app-as-a-sidecar-509j) - Practical guide with examples
- [Tauri — How to Start/Stop a sidecar - Medium](https://medium.com/@samuelint/tauri-how-to-start-stop-a-sidecar-and-pipe-sidecar-stdout-stderr-to-app-logs-from-rust-8f81a92111ad) - stdout/stderr parsing patterns
- [Kill process on exit - Tauri Discussion](https://github.com/tauri-apps/tauri/discussions/3273) - Process cleanup patterns, orphan prevention
- [Node.js: How to Get Server's Port Dynamically](https://www.w3tutorials.net/blog/nodejs-how-to-get-the-server-s-port/) - Port 0 and server.address() usage
- [Vite-tsconfig-paths Plugin](https://www.npmjs.com/package/vite-tsconfig-paths) - Path alias resolution in Vite

### Tertiary (LOW confidence)
- [tauri-plugin-cors-fetch](https://crates.io/crates/tauri-plugin-cors-fetch/1.0.1) - Third-party CORS plugin (official HTTP plugin preferred)
- Community discussions about pkg native module limitations - Known issue, no definitive solution

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries verified via official docs, versions current as of Jan 2026
- Architecture: HIGH - Patterns from official Tauri 2.0 docs, Hono official guide, verified examples
- Pitfalls: MEDIUM-HIGH - Mix of official warnings, GitHub issues, and community-reported experiences

**Research date:** 2026-01-31
**Valid until:** 2026-03-02 (30 days - Tauri 2.x stable, Hono stable, Tailwind v4 stable)
