import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { APP_NAME } from "@aios/shared";

const app = new Hono();

app.get("/health", (c) => c.json({ status: "ok", app: APP_NAME }));

const port = Number(process.env.PORT) || 3001;

serve({ fetch: app.fetch, port }, (info) => {
  console.log(`${APP_NAME} backend running on http://localhost:${info.port}`);
});

export type AppType = typeof app;
