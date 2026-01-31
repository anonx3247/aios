# Domain Pitfalls: Desktop AI Agent Launcher

**Domain:** AI-powered desktop agent launcher (Tauri + background agents + MCP)
**Researched:** 2026-01-31
**Confidence:** MEDIUM-HIGH (verified with recent sources, some domain-specific areas lack official documentation)

## Executive Summary

Desktop AI agent launchers face unique challenges at the intersection of native app development, AI agent orchestration, background process management, and user experience. The most critical pitfalls cluster around **sidecar lifecycle management** (orphaned processes), **security boundaries** (dynamic UI execution, agent permissions), **reliability** (event hook failures, webhook delivery), and **performance** (memory growth, notification spam, indexing overhead).

This research draws from 2026 security advisories, recent Tauri GitHub issues, MCP security assessments, and production deployment experiences.

---

## Critical Pitfalls

Mistakes that cause rewrites, security incidents, or complete system failures.

### Pitfall 1: Sidecar Process Orphaning

**What goes wrong:** Tauri sidecar processes (Node backend, agent harness) continue running after the main app exits, polluting the user's machine with orphaned processes consuming CPU/memory.

**Why it happens:**
- Tauri doesn't automatically kill child processes on app exit
- Cleanup handlers fail silently during forced termination (kill -9, crash, AutoCAD palette close)
- No standardized health checks or lifecycle management in Tauri core
- Developers assume process.on('exit') is reliable (it's not for forced kills)

**Consequences:**
- User's machine accumulates zombie processes over time
- Battery drain and system slowdown
- Lost trust — users notice processes in Activity Monitor
- Data corruption if sidecar was mid-write during orphaning

**Prevention:**
```typescript
// DON'T: Assume cleanup handlers will run
process.on('exit', () => sidecar.kill());

// DO: Implement heartbeat + watchdog pattern
// Main app sends heartbeat every 5s
// Sidecar self-terminates after 15s without heartbeat
```

**Rust-side prevention:**
```rust
// Track sidecar PID explicitly
// On app drop, force kill with timeout
impl Drop for AppState {
    fn drop(&mut self) {
        if let Some(pid) = self.sidecar_pid {
            // Force kill with 2s timeout
            let _ = Command::new("kill")
                .args(["-9", &pid.to_string()])
                .spawn();
        }
    }
}
```

**Detection:**
- Activity Monitor showing multiple instances of node backend
- Port conflicts on app restart (localhost:3001 already in use)
- User reports of battery drain
- System resource monitoring shows processes without parent

**Phase mapping:**
- **Phase 1 (MVP):** Implement basic cleanup handlers
- **Phase 2:** Add heartbeat pattern and watchdog
- **Phase 3:** Production hardening with process monitoring

**Sources:**
- [Tauri sidecar orphan process issue #1896](https://github.com/tauri-apps/tauri/issues/1896)
- [Node backend running in background #8689](https://github.com/tauri-apps/tauri/issues/8689)
- [Sidecar Lifecycle Management Plugin proposal #3062](https://github.com/tauri-apps/plugins-workspace/issues/3062)

---

### Pitfall 2: Dynamic UI Sandbox Escape

**What goes wrong:** Agents write malicious JSX that escapes sandbox, executes arbitrary code, accesses filesystem, or exfiltrates data.

**Why it happens:**
- Using `new Function()` or `eval()` to execute agent-generated JSX
- vm2 library has critical sandbox escape vulnerability (CVE-2026-22709, CVSS 9.8)
- `with` statement and Function constructor can bypass sandboxing
- Babel standalone transforms JSX in-browser but doesn't sandbox execution
- Agent has access to process.env, require(), or Node APIs in component context

**Consequences:**
- Agent reads API keys from localStorage
- Agent writes to arbitrary files via Node fs
- Agent spawns child processes
- Data exfiltration to external servers
- Complete system compromise if agent has host-mode process-mcp access

**Prevention:**
```typescript
// DON'T: Use vm2 (has critical CVE in 2026)
const vm = require('vm2');
const result = new vm.VM().run(agentCode);

// DON'T: Use new Function with agent code
const Component = new Function('React', agentJSX);

// DO: Use isolated iframe with sandboxed attributes
<iframe
  sandbox="allow-scripts"
  srcDoc={transformedComponent}
  // No allow-same-origin = isolated from parent
/>

// DO: Web Worker isolation for computation
const worker = new Worker(
  URL.createObjectURL(new Blob([componentCode]))
);
```

**Allowlist approach:**
```typescript
// Only expose safe React APIs
const SafeReact = {
  createElement: React.createElement,
  useState: React.useState,
  useEffect: React.useEffect,
  // NO useRef, useImperativeHandle, etc.
};

// Render with restricted scope
const Component = compileJSX(agentCode, { React: SafeReact });
```

**Detection:**
- Unexpected network requests from renderer process
- File access outside expected directories
- Process spawns not from main app
- Security audit: check dynamic component imports
- Code review: search for `eval`, `new Function`, `vm.runInContext`

**Phase mapping:**
- **Phase 1:** Use iframe sandbox for all dynamic components
- **Phase 2:** Implement component allowlist and API restrictions
- **Phase 3:** Add runtime monitoring for escape attempts

**Sources:**
- [CVE-2026-22709: Critical vm2 sandbox escape](https://www.endorlabs.com/learn/cve-2026-22709-critical-sandbox-escape-in-vm2-enables-arbitrary-code-execution)
- [JavaScript Sandboxing deep dive](https://leapcell.medium.com/a-deep-dive-into-javascript-sandboxing-bbb0773a8633)
- [Dynamic React JSX rendering without build step](https://dev.to/mirshahreza/dynamic-react-rendering-remote-jsx-without-a-build-step-2aal)

---

### Pitfall 3: Semantic Privilege Escalation

**What goes wrong:** User with limited permissions asks agent to perform task. Agent uses its broad permissions to access data or execute actions beyond the user's intent, bypassing authorization boundaries.

**Why it happens:**
- Agent granted broad permissions (file system, Docker, webhooks, process execution) to handle diverse requests
- Traditional access control only checks "can agent do X?" not "should agent do X for this task?"
- No semantic authorization layer to evaluate action-task alignment
- Tools can be combined in unexpected ways (read API key file → exfiltrate via webhook)
- Agent becomes a privilege escalation intermediary

**Consequences:**
- Agent reads sensitive files outside task scope
- Agent transfers funds or deletes data without explicit user approval
- Cascading failures from misaligned tool combinations
- Regulatory violation (GDPR, EU AI Act requires human oversight for high-risk actions)
- Undetectable by traditional RBAC (agent has permission, but action is wrong)

**Prevention:**
```typescript
// High-risk actions require explicit confirmation
const HIGH_RISK_TOOLS = [
  'delete_file',
  'execute_shell_command',
  'send_webhook',
  'docker_run'
];

async function executeTool(toolName, params, context) {
  if (HIGH_RISK_TOOLS.includes(toolName)) {
    // Send OS notification asking user to approve
    const approved = await requestUserApproval({
      tool: toolName,
      params: params,
      taskContext: context.originalRequest
    });
    if (!approved) {
      throw new Error('User denied high-risk action');
    }
  }
  return await tools[toolName](params);
}
```

**Scope-based authorization:**
```typescript
// Map task intent to allowed tool scope
const taskScope = analyzeTaskIntent(userRequest);
// "Search for Python files" → scope: read_file, list_directory
// "Deploy app" → scope: docker_run, write_file, execute_shell

// Reject tool calls outside scope
if (!taskScope.allowedTools.includes(toolCall.tool)) {
  throw new Error(`Tool ${toolCall.tool} not in scope for task`);
}
```

**Detection:**
- Audit log showing tool combinations that don't align with user request
- File access patterns deviating from task description
- Webhook/network calls in tasks that don't mention external communication
- User reports: "Agent did something I didn't ask for"

**Phase mapping:**
- **Phase 1:** Implement high-risk tool confirmation (OS notifications)
- **Phase 2:** Build semantic scope analyzer
- **Phase 3:** Full audit logging and policy engine

**Sources:**
- [AI Agents becoming authorization bypass paths](https://thehackernews.com/2026/01/ai-agents-are-becoming-privilege.html)
- [Semantic privilege escalation threat](https://acuvity.ai/semantic-privilege-escalation-the-agent-security-threat-hiding-in-plain-sight/)
- [OWASP AI Agent Security Top 10 2026](https://medium.com/@oracle_43885/owasps-ai-agent-security-top-10-agent-security-risks-2026-fc5c435e86eb)

---

### Pitfall 4: Event Hook Silent Failures

**What goes wrong:** Cron jobs, file watchers, and webhook listeners fail silently. User expects agent to run on schedule/event but nothing happens. No error, no notification, no retry.

**Why it happens:**
- Cron has no execution guarantee (system down = missed job, no catch-up)
- File watchers don't fire in git repos with no commits
- Webhook endpoints return 500, but no retry logic exists
- No monitoring or alerting for hook failures
- Assumes "if I set it up, it works"

**Consequences:**
- User creates "daily backup" cron → system down during backup window → no backup, no alert
- File watcher monitoring ~/Documents → never triggers → user assumes it's working
- Critical webhook (payment notification) fails → silent data loss
- Lost trust: "I thought agents were running"

**Prevention:**

**Cron reliability:**
```typescript
// DON'T: Just schedule and forget
cron.schedule('0 2 * * *', backupTask);

// DO: Track execution, detect missed runs
interface CronRun {
  scheduledAt: Date;
  executedAt: Date | null;
  status: 'success' | 'failure' | 'missed';
  error?: string;
}

// On app startup: check for missed runs
const lastRun = await db.getLastCronRun('daily-backup');
const expectedRun = getLastScheduledTime('0 2 * * *');
if (expectedRun > lastRun.scheduledAt) {
  // Missed run detected
  await sendNotification('Missed scheduled task: daily-backup');
  await executeCatchup('daily-backup');
}
```

**Webhook reliability with exponential backoff:**
```typescript
async function deliverWebhook(url, payload, attempt = 0) {
  try {
    const response = await fetch(url, {
      method: 'POST',
      body: JSON.stringify(payload),
      timeout: 10000 // 10s timeout
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    await db.markWebhookDelivered(payload.id);
  } catch (error) {
    if (attempt < 5) {
      // Exponential backoff with jitter
      const delay = Math.min(1000 * Math.pow(2, attempt) + Math.random() * 1000, 60000);
      setTimeout(() => deliverWebhook(url, payload, attempt + 1), delay);
    } else {
      // Dead letter queue after 5 attempts
      await db.moveToDeadLetterQueue(payload);
      await sendNotification(`Webhook failed after 5 attempts: ${url}`);
    }
  }
}
```

**File watcher with health checks:**
```typescript
const watcher = chokidar.watch('/path/to/watch');
let lastEvent = Date.now();

// Health check: ensure watcher is alive
setInterval(() => {
  if (Date.now() - lastEvent > 3600000) { // 1 hour
    // Create test file to verify watcher works
    fs.writeFileSync('/path/to/watch/.health-check', 'test');
  }
}, 600000); // Check every 10 minutes

watcher.on('change', (path) => {
  lastEvent = Date.now();
  if (path.includes('.health-check')) return;
  // Handle real event
});
```

**Detection:**
- Audit expected vs actual executions
- Health check files for watchers
- Webhook delivery dashboard showing failures
- User report: "My scheduled task didn't run"

**Phase mapping:**
- **Phase 1:** Basic cron with execution logging
- **Phase 2:** Missed run detection + catch-up logic
- **Phase 3:** Webhook retry + DLQ, file watcher health checks

**Sources:**
- [Cron has no execution guarantee](https://uptimerobot.com/knowledge-hub/cron-monitoring/cron-job-guide/)
- [File watcher events never fire in git repos](https://github.com/sst/opencode/issues/5087)
- [Webhook retry best practices](https://www.svix.com/resources/webhook-best-practices/retries/)
- [How to monitor cron jobs in 2026](https://dev.to/cronmonitor/how-to-monitor-cron-jobs-in-2026-a-complete-guide-28g9)

---

### Pitfall 5: Vector Embedding Drift on Model Upgrade

**What goes wrong:** Agent memory uses vector embeddings for semantic search. Upgrade embedding model → all existing vectors incompatible → memory search broken → agent can't recall past information.

**Why it happens:**
- Embedding model upgraded (OpenAI text-embedding-3-small v1 → v2)
- Vector dimensions change (768 → 1024)
- Semantic space shifts (same text maps to different vectors)
- Re-encoding entire corpus requires significant compute
- Rebuilding ANN index causes downtime
- No migration strategy planned

**Consequences:**
- Agent memory search returns wrong results or fails entirely
- "Agent forgot everything" after model upgrade
- Forced to rebuild entire vector database (hours/days for large corpora)
- Downtime during re-indexing
- Lost semantic relationships between old and new memories

**Prevention:**

**Drift-Adapter pattern (95-99% recall recovery):**
```typescript
// Lightweight transformation layer maps new embeddings to old space
// Avoids full re-encoding

// Train adapter on sample data
const adapter = trainDriftAdapter({
  oldModel: 'text-embedding-3-small-v1',
  newModel: 'text-embedding-3-small-v2',
  sampleSize: 1000 // Representative sample
});

// Query using new model, map to old space
async function semanticSearch(query: string) {
  const newEmbedding = await embedNew(query);
  const mappedEmbedding = adapter.transform(newEmbedding);
  // Use existing ANN index with mapped embedding
  return await vectorDB.search(mappedEmbedding);
}

// Gradual migration: re-encode in background
backgroundJob(async () => {
  const oldVectors = await db.getVectorsForReencoding(batchSize: 100);
  const newVectors = await reencodeWithNewModel(oldVectors);
  await db.updateVectors(newVectors);
});
```

**Version pinning with migration plan:**
```typescript
interface VectorConfig {
  model: string;
  version: string;
  dimensions: number;
  migratedAt?: Date;
}

// Pin embedding model version
const config: VectorConfig = {
  model: 'text-embedding-3-small',
  version: 'v1',
  dimensions: 768
};

// Detection: alert when models drift
if (config.version !== latestVersion) {
  console.warn('Embedding model drift detected');
  // Trigger migration workflow
}
```

**Monitoring drift:**
```typescript
// Test query recall quality
async function detectDrift() {
  const testQueries = [/* representative queries */];
  const recall = await measureRecall(testQueries);

  if (recall < 0.95) {
    await sendNotification('Vector search recall degraded');
    // Trigger re-encoding or adapter training
  }
}
```

**Detection:**
- Semantic search returns unexpected results
- Recall metrics drop below threshold
- User reports: "Agent can't remember recent information"
- Embedding model version mismatch in logs

**Phase mapping:**
- **Phase 1:** Pin embedding model version explicitly
- **Phase 2:** Implement drift monitoring and alerts
- **Phase 3:** Build Drift-Adapter for zero-downtime upgrades

**Sources:**
- [Drift-Adapter for near zero-downtime embedding upgrades](https://arxiv.org/html/2509.23471)
- [What is embedding drift and how to detect it](https://zilliz.com/ai-faq/what-is-embedding-drift-and-how-do-i-detect-it)
- [Impact of embedding drift](https://milvus.io/ai-quick-reference/what-is-the-impact-of-embedding-drift-and-how-do-i-manage-it)

---

## Moderate Pitfalls

Mistakes that cause delays, technical debt, or degraded UX.

### Pitfall 6: MCP Server Command Injection

**What goes wrong:** Agent passes unsanitized user input to MCP tool → shell command injection → arbitrary code execution on host or in Docker container.

**Why it happens:**
- MCP servers like process-mcp execute shell commands
- Tool parameters constructed with string concatenation
- No input validation or sanitization
- Equixly security audit (2025) found 43% of MCP implementations vulnerable

**Consequences:**
- User input like `; rm -rf /` executed on host system
- Docker container escape via malicious command
- Data exfiltration through command substitution
- Host compromise if process-mcp in host mode

**Prevention:**
```typescript
// DON'T: String concatenation for shell commands
const cmd = `ls ${userInput}`;

// DO: Use parameterized execution
import { spawn } from 'child_process';
const result = spawn('ls', [userInput], {
  shell: false // Disable shell interpretation
});

// DO: Allowlist validation
function validateFilename(input: string): string {
  if (!/^[a-zA-Z0-9_\-\.\/]+$/.test(input)) {
    throw new Error('Invalid filename');
  }
  return input;
}
```

**Detection:**
- Security audit: search for `exec`, `eval`, template literals in tool code
- Input containing shell metacharacters (`;`, `|`, `&`, `$()`, backticks)
- Unexpected process spawns in system logs

**Phase mapping:**
- **Phase 1:** Parameterized execution for all process-mcp calls
- **Phase 2:** Input validation library with allowlist patterns
- **Phase 3:** Automated security scanning for injection vulnerabilities

**Sources:**
- [MCP security risks and controls](https://www.redhat.com/en/blog/model-context-protocol-mcp-understanding-security-risks-and-controls)
- [43% of MCP implementations vulnerable to command injection](https://www.cdata.com/blog/2026-year-enterprise-ready-mcp-adoption)

---

### Pitfall 7: OS Notification Spam → User Disables All Notifications

**What goes wrong:** Agent sends too many notifications (progress updates, every tool call, verbose errors) → user annoyed → disables notifications entirely → misses critical alerts (agent needs input, task completed).

**Why it happens:**
- No notification priority system
- Every event triggers notification
- No grouping or rate limiting
- Marketing mindset ("more engagement = better")
- Forgetting users have aggressive Focus modes in 2026

**Consequences:**
- User permanently silences app (macOS Settings → Notifications → AIOS → Off)
- Agent needs user input but notification never seen
- Task completion notification missed
- App perceived as spam/annoying
- Uninstall

**Prevention:**

**Priority-based notifications:**
```typescript
enum NotificationPriority {
  CRITICAL = 'critical',  // Agent needs input, error
  HIGH = 'high',          // Task completed
  MEDIUM = 'medium',      // Progress milestone
  LOW = 'low'             // Background info
}

// Only send CRITICAL and HIGH by default
async function sendNotification(
  message: string,
  priority: NotificationPriority
) {
  const userSettings = await getUserNotificationSettings();

  if (priority === NotificationPriority.LOW && !userSettings.enableLowPriority) {
    return; // Skip low priority
  }

  if (priority === NotificationPriority.MEDIUM && !userSettings.enableProgress) {
    return; // Skip progress updates
  }

  // Always send CRITICAL and HIGH
  await platformNotification(message, {
    priority,
    sound: priority === NotificationPriority.CRITICAL
  });
}
```

**Rate limiting:**
```typescript
// Don't send more than 3 notifications per 10 minutes
const recentNotifications: Date[] = [];

function canSendNotification(): boolean {
  const now = Date.now();
  const tenMinutesAgo = now - 10 * 60 * 1000;

  recentNotifications = recentNotifications.filter(t => t > tenMinutesAgo);

  if (recentNotifications.length >= 3) {
    // Queue notification for later
    return false;
  }

  recentNotifications.push(now);
  return true;
}
```

**Grouping:**
```typescript
// Group related notifications
// "Agent completed 5 tasks" instead of 5 separate notifications
const completedTasks: string[] = [];
const groupTimeout = setTimeout(() => {
  if (completedTasks.length > 0) {
    sendNotification(
      `Completed ${completedTasks.length} tasks`,
      NotificationPriority.HIGH
    );
  }
}, 30000); // Group within 30s window
```

**Detection:**
- User disables notifications in OS settings
- High notification frequency in logs (>5 per hour)
- User feedback: "Too many notifications"
- Analytics: notification permission revoked

**Phase mapping:**
- **Phase 1:** Only notify on task completion and errors
- **Phase 2:** Add priority system and user settings
- **Phase 3:** Implement rate limiting and grouping

**Sources:**
- [App push notification best practices 2026](https://appbot.co/blog/app-push-notifications-2026-best-practices/)
- [Push notification best practices (ultimate guide)](https://reteno.com/blog/push-notification-best-practices-ultimate-guide-for-2026/)

---

### Pitfall 8: SQLite FTS5 Performance Degradation at Scale

**What goes wrong:** Agent memory using SQLite FTS5 for keyword search performs well initially → memory grows to 100K+ entries → search becomes slow (seconds instead of milliseconds) → agent waits for memory queries → user perceives lag.

**Why it happens:**
- FTS5 not optimized for 100K+ documents without tuning
- No index optimization or VACUUM
- Full table scans on unindexed columns
- BM25 ranking on every query is expensive
- No query result caching
- Writing to FTS5 tables without transactions

**Consequences:**
- Memory search takes 2-5 seconds → agent response delayed
- UI freezes during search
- Battery drain from repeated full scans
- User frustration: "Why is it so slow?"

**Prevention:**

**Optimize FTS5 configuration:**
```sql
-- Use prefix indexing for faster prefix queries
CREATE VIRTUAL TABLE memories_fts USING fts5(
  content,
  tokenize='porter', -- Stemming
  prefix='2 3'       -- Enable prefix search
);

-- Separate content table from FTS index
CREATE TABLE memories(id INTEGER PRIMARY KEY, content TEXT);
CREATE VIRTUAL TABLE memories_fts USING fts5(
  content,
  content=memories,
  content_rowid=id
);
```

**Regular optimization:**
```typescript
// Run after bulk inserts
await db.exec("INSERT INTO memories_fts(memories_fts) VALUES('optimize')");

// Periodic VACUUM (reclaim space, defragment)
setInterval(async () => {
  await db.exec("VACUUM");
}, 7 * 24 * 60 * 60 * 1000); // Weekly
```

**Hybrid search (FTS5 + vector):**
```typescript
// Narrow search with FTS5, re-rank with vectors
async function hybridSearch(query: string, limit: number) {
  // FTS5 gets top 100 candidates (fast)
  const candidates = await db.all(`
    SELECT id, content, rank
    FROM memories_fts
    WHERE memories_fts MATCH ?
    ORDER BY rank
    LIMIT 100
  `, [query]);

  // Vector search re-ranks top 100 (smaller search space)
  const queryEmbedding = await embed(query);
  const ranked = candidates
    .map(c => ({
      ...c,
      similarity: cosineSimilarity(queryEmbedding, c.embedding)
    }))
    .sort((a, b) => b.similarity - a.similarity)
    .slice(0, limit);

  return ranked;
}
```

**Query caching:**
```typescript
const queryCache = new Map<string, SearchResult[]>();

async function cachedSearch(query: string) {
  if (queryCache.has(query)) {
    return queryCache.get(query);
  }

  const results = await db.search(query);
  queryCache.set(query, results);

  // Invalidate cache after 5 minutes
  setTimeout(() => queryCache.delete(query), 5 * 60 * 1000);

  return results;
}
```

**Detection:**
- Memory search queries taking >500ms
- CPU spikes during FTS5 queries
- User reports of lag when agent searches memory
- Database file size growing without performance tuning

**Phase mapping:**
- **Phase 1:** Basic FTS5 with porter tokenization
- **Phase 2:** Hybrid search with vector re-ranking
- **Phase 3:** Query caching and periodic optimization

**Sources:**
- [SQLite FTS5 extension documentation](https://sqlite.org/fts5.html)
- [FTS5 performance regression in 3.51.0](https://sqlite.org/forum/info/76e9f978b36223149ddbb76c05482c49daf0501008d49b931e1039bc7ea28206)
- [Hybrid FTS5 + vector search guide](https://alexgarcia.xyz/blog/2024/sqlite-vec-hybrid-search/index.html)

---

### Pitfall 9: Localhost Port Conflicts with Other Desktop Apps

**What goes wrong:** Node backend hardcoded to localhost:3001 → conflicts with another app using same port → AIOS won't start → cryptic error "EADDRINUSE".

**Why it happens:**
- Hardcoded port (3001) in both Node backend and frontend
- No automatic port fallback
- Common ports (3000, 3001, 3002) used by dev servers, Ollama, databases
- User runs multiple desktop apps simultaneously

**Consequences:**
- App fails to start with unclear error
- Frontend can't connect to backend (port mismatch)
- User forced to manually kill other process
- Poor developer experience

**Prevention:**

**Dynamic port allocation:**
```typescript
import getPort from 'get-port';

// Find available port starting from 3001
const port = await getPort({ port: [3001, 3002, 3003, 3004, 3005] });

// Save to runtime config for frontend to discover
await writeFile('.runtime-config.json', JSON.stringify({ backendPort: port }));

server.listen(port, () => {
  console.log(`Backend listening on port ${port}`);
});
```

**Frontend discovery:**
```typescript
// Read runtime config instead of hardcoded URL
const config = await readFile('.runtime-config.json', 'utf-8');
const { backendPort } = JSON.parse(config);

const API_URL = `http://localhost:${backendPort}`;
```

**Tauri IPC alternative:**
```typescript
// Avoid HTTP altogether for local communication
// Use Tauri IPC to proxy to Node backend

// Backend registers on random port
const port = await getPort();

// Tauri command proxies requests
#[tauri::command]
async fn proxy_to_backend(
  endpoint: String,
  body: String
) -> Result<String> {
  let port = APP_STATE.backend_port;
  let url = format!("http://localhost:{}/{}", port, endpoint);
  // Forward request
}

// Frontend uses Tauri command instead of fetch
const response = await invoke('proxy_to_backend', {
  endpoint: 'api/chat',
  body: JSON.stringify(message)
});
```

**Detection:**
- Error message "EADDRINUSE" in logs
- App startup failure
- Frontend errors "Failed to connect to backend"
- User report: "App won't start"

**Phase mapping:**
- **Phase 1:** Dynamic port allocation with get-port
- **Phase 2:** Frontend discovery via runtime config
- **Phase 3:** Consider Tauri IPC proxy as alternative

**Sources:**
- [How I manage localhost port conflicts with AI agent](https://block.github.io/goose/blog/2025/05/22/manage-local-host-conflicts-with-goose/)
- [Port management in Node.js](https://dev.to/sudiip__17/-port-management-in-nodejs-running-multiple-servers-like-a-pro-ilc)

---

### Pitfall 10: Cross-Platform Path Handling (macOS → Windows/Linux)

**What goes wrong:** Agent launcher uses hardcoded macOS paths (`~/Library/Application Support/AIOS`) → app ported to Windows/Linux → file operations fail → crash or data loss.

**Why it happens:**
- Hardcoded macOS-specific paths
- Using `/` separators instead of `path.join()`
- Case-sensitive filesystem assumptions (macOS is case-insensitive by default)
- User directory expansion (`~`) not portable
- Different app data locations per OS

**Consequences:**
- Windows: App writes to wrong directory or crashes
- Linux: Permission errors (~/Library doesn't exist)
- File watchers fail on different filesystem events
- Notification APIs differ (macOS Notification Center vs Windows Toast)

**Prevention:**

**Use Tauri's app path APIs:**
```rust
use tauri::Manager;

// Get app data directory (cross-platform)
let app_data = app.path_resolver()
  .app_data_dir()
  .expect("Failed to resolve app data directory");
// macOS: ~/Library/Application Support/AIOS
// Windows: C:\Users\{username}\AppData\Roaming\AIOS
// Linux: ~/.config/AIOS

// Get app cache directory
let cache = app.path_resolver().app_cache_dir();
// macOS: ~/Library/Caches/AIOS
// Windows: C:\Users\{username}\AppData\Local\AIOS\cache
```

**Path handling:**
```typescript
import path from 'path';

// DON'T: Hardcoded separator
const filePath = `${baseDir}/data/memories.db`;

// DO: Use path.join
const filePath = path.join(baseDir, 'data', 'memories.db');

// DO: Normalize user input paths
const userPath = path.normalize(userInput);
```

**OS-specific feature detection:**
```typescript
import os from 'os';

const platform = os.platform();

if (platform === 'darwin') {
  // macOS-specific: global shortcut, Notification Center
} else if (platform === 'win32') {
  // Windows-specific: Toast notifications
} else if (platform === 'linux') {
  // Linux-specific: libnotify
}
```

**Detection:**
- App crashes on Windows/Linux
- File not found errors in logs
- User reports: "Can't save settings"
- Path strings with hardcoded `/` in error messages

**Phase mapping:**
- **Phase 1:** Use Tauri path APIs from day 1 (macOS-first but portable)
- **Phase 2:** Test on Windows/Linux VMs before each release
- **Phase 3:** CI/CD builds for all platforms

**Sources:**
- [Cross-platform desktop app development](https://leadwebpraxis.com/cross-platform-compatibility-making-desktop-apps-work-on-windows-macos-and-linux/)
- [Tauri path resolver documentation](https://v2.tauri.app/develop/)

---

## Minor Pitfalls

Mistakes that cause annoyance but are fixable without major rewrites.

### Pitfall 11: Spotlight Indexing CPU Spike on Startup

**What goes wrong:** macOS Spotlight (corespotlightd) indexes AIOS's app data directory → CPU spike to 100%+ → system slowdown → battery drain.

**Why it happens:**
- App writes many small files (SQLite journal, vector embeddings, logs)
- Spotlight indexes everything in ~/Library by default
- Large batch operations trigger re-indexing
- Corrupted Spotlight index

**Consequences:**
- System lag on AIOS startup
- Battery drain from corespotlightd
- User blames AIOS for poor performance

**Prevention:**
```bash
# Exclude app data from Spotlight indexing
# Add to app installation script
touch ~/Library/Application\ Support/AIOS/.metadata_never_index

# Programmatic approach (requires admin)
mdutil -i off ~/Library/Application\ Support/AIOS
```

**Detection:**
- Activity Monitor shows corespotlightd high CPU after AIOS starts
- User reports system slowdown after launching app

**Phase mapping:**
- **Phase 2:** Add .metadata_never_index on first run

**Sources:**
- [corespotlightd high CPU on Mac](https://macsecurity.net/view/643-corespotlightd-high-cpu-process-on-mac)
- [Fixing high CPU usage by Spotlight](https://techgarden.alphasmanifesto.com/mac/Fixing-high-CPU-usage-by-Spotlight)

---

### Pitfall 12: Agent Memory Leak from Uncleared Conversation History

**What goes wrong:** Agent runtime keeps full conversation history in memory → long-running tasks accumulate 10K+ messages → memory usage grows from 100MB to 2GB → app slows down or crashes.

**Why it happens:**
- No message pruning strategy
- Streaming responses append to in-memory array
- Tool call results stored indefinitely
- Assumes memory is infinite

**Consequences:**
- App memory grows unbounded
- Electron/Tauri renderer process crashes (OOM)
- System swap thrashing
- User perceives app as "heavy"

**Prevention:**
```typescript
interface ConversationManager {
  maxMessages: number;

  addMessage(msg: Message) {
    this.messages.push(msg);

    // Prune old messages (keep last 100)
    if (this.messages.length > this.maxMessages) {
      const toRemove = this.messages.length - this.maxMessages;
      this.messages.splice(0, toRemove);
    }
  }
}

// Summarize and discard old context
async function compressConversationHistory() {
  if (messages.length > 50) {
    const summary = await summarizeMessages(messages.slice(0, -20));
    messages = [
      { role: 'system', content: `Previous context: ${summary}` },
      ...messages.slice(-20) // Keep recent 20
    ];
  }
}
```

**Detection:**
- Memory usage growing over time
- App slowdown after long sessions
- Crash reports with OOM errors

**Phase mapping:**
- **Phase 1:** Max message limit (100 messages)
- **Phase 2:** Automatic summarization for long conversations

**Sources:**
- [Persistent agent memory management](https://medium.com/@dharamai2024/persistent-storage-in-adk-building-memory-agents-with-sqlite-part-5-c0a2e4a058a5)
- [LangGraph memory management guide](https://pub.towardsai.net/understanding-memory-management-in-langgraph-a-practical-guide-for-genai-students-b3642c9ea7e1)

---

### Pitfall 13: Docker Mode Falls Back to Host Without Warning

**What goes wrong:** Agent uses process-mcp Docker mode for sandboxed execution → Docker not running → process-mcp silently falls back to host mode → agent executes dangerous commands on real system.

**Why it happens:**
- No Docker health check before execution
- Automatic fallback to host mode on Docker failure
- User unaware of execution mode
- No UI indication of Docker vs host

**Consequences:**
- `rm -rf /important-data` runs on host instead of container
- Agent installs packages globally instead of in sandbox
- User expects sandboxing but gets none
- Data loss or system damage

**Prevention:**
```typescript
// Require explicit mode selection, no fallback
enum ExecutionMode {
  DOCKER = 'docker',
  HOST = 'host'
}

async function executeCommand(cmd: string, mode: ExecutionMode) {
  if (mode === ExecutionMode.DOCKER) {
    // Check Docker availability
    const dockerRunning = await checkDockerHealth();
    if (!dockerRunning) {
      throw new Error('Docker not available. Start Docker or switch to host mode.');
    }
  }

  if (mode === ExecutionMode.HOST) {
    // Require user confirmation for host mode
    const confirmed = await confirmHostExecution(cmd);
    if (!confirmed) {
      throw new Error('Host execution cancelled by user');
    }
  }

  return await processMCP.execute(cmd, mode);
}
```

**UI indication:**
```typescript
// Show execution mode in agent board UI
<TaskHeader>
  <TaskTitle>{task.name}</TaskTitle>
  <ExecutionModeBadge mode={task.mode}>
    {task.mode === 'docker' ? '🐳 Sandboxed' : '⚠️ Host Mode'}
  </ExecutionModeBadge>
</TaskHeader>
```

**Detection:**
- User expects sandboxed execution but sees host filesystem changes
- Commands succeed even when Docker not running
- Logs show fallback to host mode

**Phase mapping:**
- **Phase 1:** Explicit mode selection, no automatic fallback
- **Phase 2:** Docker health check and user warnings
- **Phase 3:** UI mode indicator

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| **Phase 1: Spotlight Launcher** | Port conflicts with other apps | Dynamic port allocation with get-port |
| **Phase 1: Background Agents** | Orphaned sidecar processes | Implement cleanup handlers and heartbeat |
| **Phase 2: Dynamic UI Rendering** | Sandbox escape vulnerabilities | Use iframe sandbox, never vm2 or new Function |
| **Phase 2: Agent Memory** | FTS5 performance at scale | Hybrid search (FTS5 + vector re-ranking) |
| **Phase 2: OS Notifications** | Notification spam → user disables | Priority system, rate limiting |
| **Phase 3: Event Hooks (Cron)** | Silent failures on missed runs | Execution tracking, catch-up logic |
| **Phase 3: Event Hooks (Webhooks)** | No retry on delivery failure | Exponential backoff, DLQ after 5 attempts |
| **Phase 3: Event Hooks (File Watchers)** | Watcher doesn't fire in git repos | Health checks, test file pattern |
| **Phase 3: Agent Permissions** | Semantic privilege escalation | High-risk tool confirmation, scope analysis |
| **All Phases: MCP Integration** | Command injection vulnerabilities | Parameterized execution, input validation |
| **All Phases: Cross-Platform** | macOS-specific paths break on Windows/Linux | Tauri path APIs from day 1 |

---

## Domain-Specific Validation Checklist

Before releasing each phase:

**Security:**
- [ ] Dynamic UI components rendered in sandboxed iframe
- [ ] No usage of vm2, eval(), or new Function with agent code
- [ ] High-risk tools require user confirmation via OS notification
- [ ] MCP tool parameters validated and sanitized (no command injection)
- [ ] Agent permissions scoped to task intent

**Reliability:**
- [ ] Sidecar processes have cleanup handlers and heartbeat watchdog
- [ ] Cron jobs track execution and detect missed runs
- [ ] Webhooks retry with exponential backoff (5 attempts max)
- [ ] File watchers have health checks
- [ ] Dead letter queue for undeliverable events

**Performance:**
- [ ] FTS5 index optimized with porter tokenization and prefix indexing
- [ ] Hybrid search (FTS5 narrows, vector re-ranks)
- [ ] Conversation history pruned or summarized (max 100 messages in memory)
- [ ] Vector embedding model version pinned
- [ ] Drift monitoring alerts when recall drops

**UX:**
- [ ] Notifications limited to CRITICAL and HIGH priority by default
- [ ] Rate limiting: max 3 notifications per 10 minutes
- [ ] Notification grouping for related events
- [ ] Execution mode (Docker vs host) visible in UI
- [ ] Port conflicts handled with dynamic allocation

**Cross-Platform:**
- [ ] All paths use Tauri path resolver APIs
- [ ] path.join() instead of hardcoded `/` separators
- [ ] OS-specific features detected at runtime
- [ ] Tested on macOS, Windows, Linux before release

---

## Confidence Assessment

| Area | Confidence | Reasoning |
|------|------------|-----------|
| **Tauri sidecar issues** | HIGH | Multiple GitHub issues, official docs, community discussions |
| **Dynamic UI security** | HIGH | Recent CVE-2026-22709, security research, sandbox guides |
| **MCP integration pitfalls** | MEDIUM-HIGH | Official security assessment, but MCP still evolving |
| **Event hook reliability** | MEDIUM-HIGH | Established patterns (cron monitoring, webhook retries) |
| **Vector/SQLite performance** | MEDIUM | Official SQLite docs, but vector DB space evolving rapidly |
| **Notification best practices** | HIGH | Recent 2026 guides, OS behavior documented |
| **Agent permission model** | MEDIUM | New domain (2026), emerging standards |
| **Cross-platform pitfalls** | HIGH | Well-documented Tauri patterns |

**Overall confidence:** MEDIUM-HIGH

Research grounded in 2026 security advisories, active GitHub issues, and production deployment experiences. Some areas (MCP security, agent permissions) are emerging domains with less historical precedent, flagged accordingly.

---

## Research Gaps & Future Investigation

**Areas needing deeper research:**

1. **Universal-agent-harness specific pitfalls** — Library is user-owned, no public documentation. Need to audit codebase for:
   - Memory leak patterns in tick-based execution
   - SQLite connection pooling issues
   - Cost tracking accuracy

2. **Process-MCP Docker escape vectors** — Need security audit:
   - Container breakout via volume mounts
   - Capability escalation in Docker mode
   - TTY/stdin streaming vulnerabilities

3. **macOS Spotlight launcher integration** — Apple's private APIs:
   - Global shortcut conflicts with other apps
   - Spotlight UI rendering performance
   - Accessibility permissions required

4. **EU AI Act compliance** — Regulatory requirements:
   - Human-in-the-loop checkpoints
   - Operational history logging
   - High-risk system classification

**Recommended approach:** Phase-specific research tasks when implementing features in those domains.

---

## Sources

**Tauri:**
- [Sidecar process orphaning #1896](https://github.com/tauri-apps/tauri/issues/1896)
- [Node backend running in background #8689](https://github.com/tauri-apps/tauri/issues/8689)
- [Sidecar Lifecycle Management Plugin #3062](https://github.com/tauri-apps/plugins-workspace/issues/3062)
- [Tauri Application Lifecycle Threats](https://v2.tauri.app/security/lifecycle/)

**Security:**
- [CVE-2026-22709: Critical vm2 sandbox escape](https://www.endorlabs.com/learn/cve-2026-22709-critical-sandbox-escape-in-vm2-enables-arbitrary-code-execution)
- [JavaScript Sandboxing deep dive](https://leapcell.medium.com/a-deep-dive-into-javascript-sandboxing-bbb0773a8633)
- [MCP security risks and controls](https://www.redhat.com/en/blog/model-context-protocol-mcp-understanding-security-risks-and-controls)
- [AI Agents becoming authorization bypass paths](https://thehackernews.com/2026/01/ai-agents-are-becoming-privilege.html)
- [Semantic privilege escalation](https://acuvity.ai/semantic-privilege-escalation-the-agent-security-threat-hiding-in-plain-sight/)
- [OWASP AI Agent Security Top 10 2026](https://medium.com/@oracle_43885/owasps-ai-agent-security-top-10-agent-security-risks-2026-fc5c435e86eb)

**Event Hooks & Reliability:**
- [Cron job guide 2026](https://uptimerobot.com/knowledge-hub/cron-monitoring/cron-job-guide/)
- [How to monitor cron jobs in 2026](https://dev.to/cronmonitor/how-to-monitor-cron-jobs-in-2026-a-complete-guide-28g9)
- [File watcher events in git repos](https://github.com/sst/opencode/issues/5087)
- [Webhook retry best practices](https://www.svix.com/resources/webhook-best-practices/retries/)
- [Webhook retry logic implementation](https://latenode.com/blog/integration-api-management/webhook-setup-configuration/how-to-implement-webhook-retry-logic)

**Database & Performance:**
- [SQLite FTS5 Extension](https://sqlite.org/fts5.html)
- [Hybrid FTS5 + vector search](https://alexgarcia.xyz/blog/2024/sqlite-vec-hybrid-search/index.html)
- [Drift-Adapter for embedding upgrades](https://arxiv.org/html/2509.23471)
- [Embedding drift detection](https://zilliz.com/ai-faq/what-is-embedding-drift-and-how-do-i-detect-it)
- [Persistent agent memory with SQLite](https://medium.com/@dharamai2024/persistent-storage-in-adk-building-memory-agents-with-sqlite-part-5-c0a2e4a058a5)

**UX & Notifications:**
- [App push notification best practices 2026](https://appbot.co/blog/app-push-notifications-2026-best-practices/)
- [Push notification best practices guide](https://reteno.com/blog/push-notification-best-practices-ultimate-guide-for-2026/)
- [Spotlight high CPU (corespotlightd)](https://macsecurity.net/view/643-corespotlightd-high-cpu-process-on-mac)

**Cross-Platform:**
- [Cross-platform desktop app development](https://leadwebpraxis.com/cross-platform-compatibility-making-desktop-apps-work-on-windows-macos-and-linux/)
- [Port conflict management](https://block.github.io/goose/blog/2025/05/22/manage-local-host-conflicts-with-goose/)
- [Port management in Node.js](https://dev.to/sudiip__17/-port-management-in-nodejs-running-multiple-servers-like-a-pro-ilc)

**MCP & Enterprise:**
- [2026: Year for Enterprise MCP Adoption](https://www.cdata.com/blog/2026-year-enterprise-ready-mcp-adoption)
- [Building effective AI agents with MCP](https://developers.redhat.com/articles/2026/01/08/building-effective-ai-agents-mcp)
