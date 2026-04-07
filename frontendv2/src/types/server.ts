// @/types/server.ts
export enum Provider {
  Vanilla = 0,
  Paper = 1,
  Fabric = 2
}

export interface CreateServerOptions {
  id: string;
  name: string;
  provider: Provider;
  version: string;
  ram_mb: number;
  port: number;
  online_mode: boolean;
}