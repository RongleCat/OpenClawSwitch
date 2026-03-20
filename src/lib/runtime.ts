import { invokeSafe } from "@/lib/desktop";

export interface RuntimeHealth {
  runtimeNotReady: boolean;
  nodeReady: boolean;
  openclawReady: boolean;
  configReady: boolean;
  localOpenclawAvailable: boolean;
  shouldSkipSetup: boolean;
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

export interface SystemGatewayStatus {
  available: boolean;
  state?: GatewayStatus["state"] | null;
  pid?: number | null;
  url?: string | null;
  message?: string | null;
}

export interface GetGatewayStatusOptions {
  includeSystemStatus?: boolean;
}

interface ResolveGatewayStatusSourcesInput {
  managed: GatewayStatus;
  reachable: boolean;
  system: SystemGatewayStatus | null;
}

const DEFAULT_GATEWAY_URL = "http://127.0.0.1:18789";
const NETWORK_FALLBACK_MESSAGE = "Detected a reachable OpenClaw gateway on the default local address.";

function normalizeGatewayUrl(url?: string | null) {
  if (!url) {
    return DEFAULT_GATEWAY_URL;
  }

  if (url.startsWith("ws://")) {
    return `http://${url.slice("ws://".length)}`;
  }

  if (url.startsWith("wss://")) {
    return `https://${url.slice("wss://".length)}`;
  }

  return url;
}

export function resolveGatewayStatusSources({
  managed,
  reachable,
  system,
}: ResolveGatewayStatusSourcesInput): GatewayStatus {
  const normalizedUrl = normalizeGatewayUrl(system?.url ?? managed.url);

  if (managed.state === "running") {
    return {
      ...managed,
      url: normalizedUrl,
    };
  }

  if (managed.state === "starting" && reachable) {
    return {
      ...managed,
      state: "running",
      url: normalizedUrl,
    };
  }

  if (system?.available && system.state === "running") {
    return {
      state: "running",
      pid: system.pid ?? managed.pid ?? null,
      url: normalizedUrl,
      message: system.message ?? managed.message ?? null,
    };
  }

  if (system?.available && system.state === "starting") {
    return {
      state: reachable ? "running" : "starting",
      pid: system.pid ?? managed.pid ?? null,
      url: normalizedUrl,
      message: system.message ?? managed.message ?? null,
    };
  }

  if (reachable) {
    return {
      state: "running",
      pid: system?.pid ?? managed.pid ?? null,
      url: normalizedUrl,
      message: system?.message ?? NETWORK_FALLBACK_MESSAGE,
    };
  }

  if (managed.state !== "stopped") {
    return {
      ...managed,
      url: normalizedUrl,
    };
  }

  return {
    ...managed,
    url: normalizedUrl,
  };
}

export async function getRuntimeHealth() {
  return invokeSafe<RuntimeHealth>("get_runtime_health", undefined, {
    runtimeNotReady: false,
    nodeReady: true,
    openclawReady: true,
    configReady: false,
    localOpenclawAvailable: false,
    shouldSkipSetup: false,
    configPath: "",
    dataDir: "",
    nodePath: "",
    openclawRoot: "",
  });
}

export async function getGatewayStatus({ includeSystemStatus = true }: GetGatewayStatusOptions = {}) {
  const [managed, reachable, system] = await Promise.all([
    invokeSafe<GatewayStatus>("get_gateway_status", undefined, {
      state: "stopped",
      pid: null,
      url: DEFAULT_GATEWAY_URL,
      message: null,
    }),
    invokeSafe<boolean>("health_check_gateway", undefined, false),
    includeSystemStatus
      ? invokeSafe<SystemGatewayStatus | null>("get_system_gateway_status", undefined, null)
      : Promise.resolve<SystemGatewayStatus | null>(null),
  ]);

  return resolveGatewayStatusSources({
    managed,
    reachable,
    system,
  });
}
