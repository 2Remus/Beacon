import { fileURLToPath } from 'url';
import { dirname } from 'path';

// Polyfill __dirname for ES Modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

import { tunnel } from 'cloudflared';
import { ChildProcess, spawn } from 'child_process';
import { bin } from 'cloudflared';
// ... rest of your code

interface TunnelStatus {
  status: 'RUNNING' | 'ERROR' | 'STOPPED';
  url?: string;
  child?: ChildProcess | null;
  error?: string;
}

interface ConnectionStatus{
  status: 'RUNNING' | 'ERROR' | 'STOPPED';
  url?: string;
  child?: ChildProcess | null;
  error?: string;
}

/**
 * Manages the lifecycle of a Cloudflare Quick Tunnel
 */
export class TunnelManager {
  private activeChild: ChildProcess | null = null;

  async start(port: number): Promise<TunnelStatus> {
    return new Promise((resolve) => {
      try {
        // We use the 'bin' path directly to spawn the process manually
        // This is exactly like your Rust Command::new()
        this.activeChild = spawn(bin, [
          'tunnel',
          '--url', `http://localhost:${port}`,
          '--no-autoupdate'
        ]);

        let urlFound = false;

        // Cloudflare outputs the URL to stderr, not stdout
        this.activeChild.stderr?.on('data', (data) => {
          const output = data.toString();

          // Regex to find the .trycloudflare.com URL in the logs
          const urlMatch = output.match(/https:\/\/[a-z0-9-]+\.trycloudflare\.com/);

          if (urlMatch && !urlFound) {
            urlFound = true;
            resolve({
              status: 'RUNNING',
              url: urlMatch[0],
              //child: this.activeChild
            });
          }
        });

        this.activeChild.on('error', (err) => {
          resolve({ status: 'ERROR', error: err.message });
        });

        // Safety timeout: if no URL in 15 seconds, fail
        setTimeout(() => {
          if (!urlFound) {
            resolve({ status: 'ERROR', error: 'Tunnel timed out waiting for URL' });
          }
        }, 15000);

      } catch (err) {
        resolve({
          status: 'ERROR',
          error: err instanceof Error ? err.message : String(err)
        });
      }
    });
  }

  stop(): TunnelStatus {
    if (this.activeChild) {
      this.activeChild.kill('SIGINT');
      this.activeChild = null;
      return { status: 'STOPPED' };
    }
    return { status: 'ERROR', error: 'No active tunnel to stop' };
  }


  async connect(remoteUrl: string, localPort: number = 25565): Promise<ConnectionStatus> {
    return new Promise((resolve) => {
      try {
        // mode: 'access tcp' makes your machine the 'receiver'
        this.activeChild = spawn(bin, [
          'access',
          'tcp',
          '--hostname', remoteUrl,
          '--listener', `127.0.0.1:${localPort}`
        ]);

        // Access mode doesn't give a URL back, it just starts listening.
        // We check if the process stays alive for 2 seconds to confirm success.
        const startTimer = setTimeout(() => {
          resolve({
            status: 'RUNNING',
            url: `localhost:${localPort}`
          });
        }, 2000);

        this.activeChild.on('error', (err) => {
          clearTimeout(startTimer);
          resolve({ status: 'ERROR', error: err.message });
        });

        // If it exits immediately, something is wrong (like the port is taken)
        this.activeChild.on('exit', (code) => {
          clearTimeout(startTimer);
          if (code !== 0) {
            resolve({ status: 'ERROR', error: `Process exited with code ${code}` });
          }
        });

      } catch (err) {
        resolve({ status: 'ERROR', error: String(err) });
      }
    });
  }
}