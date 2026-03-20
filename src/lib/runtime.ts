import { invokeSafe } from "@/lib/desktop";

export interface RuntimeHealth {
  runtimeNotReady: boolean;
  nodeReady: boolean;
  openclawReady: boolean;
  configReady: boolean;
  configPath: string;
  dataDir: string;
  nodePath: string;
  openclawRoot: string;
}

export interface GatewayStatus {
  state: "running" | "starting" | "stopped" | "error";
  pid?: number | null;
  url?: string | null;
  message?: string | null;
}

export async function getRuntimeHealth() {
  return invokeSafe<RuntimeHealth>("get_runtime_health", undefined, {
    runtimeNotReady: false,
    nodeReady: true,
    openclawReady: true,
    configReady: false,
    configPath: "",
    dataDir: "",
    nodePath: "",
    openclawRoot: "",
  });
}

export async function getGatewayStatus() {
  return invokeSafe<GatewayStatus>("get_gateway_status", undefined, {
    state: "stopped",
    pid: null,
    url: "http://127.0.0.1:18789",
    message: null,
  });
}
