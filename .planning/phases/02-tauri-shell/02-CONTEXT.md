# Phase 2: Tauri Shell - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Operational Tauri app with Rust backend providing: secrets management via keyring, MCP server process lifecycle, IPC commands callable from frontend, and frameless launcher window. SQLite persistence is handled by universal-agent-harness (not this phase).

</domain>

<decisions>
## Implementation Decisions

### Secrets management
- Use system keyring (via keyring crate) for storing secrets
- Global scope — all agents and MCP servers share the same secret set
- Settings UI for users to add/edit/delete secrets (labeled key-value fields)
- Secrets injected as environment variables when spawning MCP server processes

### MCP server process lifecycle
- Single config file for all MCP servers (similar to Claude Desktop's `claude_desktop_config.json` format)
- Config includes both required/built-in servers and user-added servers
- Lazy startup — MCP servers started on demand when an agent needs them, not at app launch
- Auto-restart on crash with backoff (3 retries then give up)
- All child processes terminate cleanly when Tauri app closes

### Agent harness
- Harness spawned per agent run (not a long-running sidecar)
- Each run gets its own process for clean isolation
- Harness handles its own SQLite persistence — Tauri doesn't manage the database schema

### Window behavior
- Frameless floating window, no title bar
- Centered on screen (both axes) — Raycast-style
- Menu bar (tray) only — no dock icon
- Dismiss on blur (click outside or Escape closes the launcher)

### IPC conventions
- snake_case command names (e.g., `get_secrets`, `start_mcp_server`)

### Claude's Discretion
- IPC payload structure and error format
- Placeholder UI content for Phase 2
- Exact window dimensions
- Keyring service name and key naming scheme
- MCP server health check approach

</decisions>

<specifics>
## Specific Ideas

- MCP server config file should feel like Claude Desktop's config — familiar to users who've set up MCP servers before
- Menu bar icon for the app (tray-only, no dock presence)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-tauri-shell*
*Context gathered: 2026-01-31*
