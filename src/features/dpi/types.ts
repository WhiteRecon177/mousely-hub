export type ConnectionType = "wired" | "wireless" | "bluetooth" | "unknown";
export type MouseSupport = "tested" | "expected" | "recognized" | "unknown";

export interface MouseCapabilities {
  battery: boolean;
  dpiRead: boolean;
  dpiWrite: boolean;
  pollingRead: boolean;
  pollingWrite: boolean;
}

export interface MouseInfo {
  connected: boolean;
  name: string;
  connection: ConnectionType;
  battery: number | null;
  charging: boolean;
  dpi: number;
  dpiMin: number;
  dpiMax: number;
  dpiStep: number;
  dpiLevels: number[];
  pollingRate: number;
  pollingOptions: number[];
  pollingSupported: number[];
  brand: string;
  capabilities: MouseCapabilities;
  support: MouseSupport;
}