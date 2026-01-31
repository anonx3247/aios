import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { APP_NAME } from "@aios/shared";

const app = new Hono();

app.get("/health", (c) =>
  c.json({
    status: "ok",
    app: APP_NAME,
    timestamp: Date.now(),
    uptime: process.uptime()
  })
);

const port = Number(process.env.PORT) || 0;

const server = serve({ fetch: app.fetch, port }, (info) => {
  console.log(`BACKEND_PORT:${info.port}`);
  console.error(`${APP_NAME} backend running on http://localhost:${info.port}`);
});

// Graceful shutdown handlers
const shutdown = () => {
  console.error("Shutting down gracefully...");
  server.close(() => {
    console.error("Server closed");
    process.exit(0);
  });
};

process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);

export type AppType = typeof app;
