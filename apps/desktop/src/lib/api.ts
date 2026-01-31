import { invoke } from '@tauri-apps/api/core';
import { fetch } from '@tauri-apps/plugin-http';

class BackendClient {
  private port: number | null = null;
  private portPromise: Promise<number> | null = null;

  async getPort(): Promise<number> {
    if (this.port !== null) return this.port;
    if (this.portPromise) return this.portPromise;

    this.portPromise = (async () => {
      try {
        const port = await invoke<number>('get_backend_port');
        return port;
      } catch (error) {
        // Development mode fallback - when running without sidecar
        console.warn(
          'Failed to get backend port from Tauri, falling back to localhost:3001',
          error
        );
        return 3001;
      }
    })();

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
    return this.call<{ status: string; timestamp: number; uptime: number }>(
      '/health'
    );
  }
}

export const backend = new BackendClient();
