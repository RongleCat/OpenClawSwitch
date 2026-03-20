import { create } from "zustand";
import { invokeSafe } from "@/lib/desktop";
import { getGatewayStatus, getRuntimeHealth, type GatewayStatus, type GetGatewayStatusOptions } from "@/lib/runtime";
import { resolveGatewayControlCommands, resolveGatewayControlMode, type GatewayControlOs } from "@/domain/gatewayControl";

export type GatewayPendingAction = "start" | "restart" | "stop" | null;
export interface GatewayRefreshOptions extends GetGatewayStatusOptions {
  silent?: boolean;
}

const resolveGatewayControlOs = (): GatewayControlOs => {
  const userAgent = navigator.userAgent.toLowerCase();
  if (userAgent.includes("windows")) return "windows";
  if (userAgent.includes("mac")) return "macos";
  return "linux";
};

const resolveGatewayControlCommandNames = async () => {
  const runtime = await getRuntimeHealth();
  const mode = resolveGatewayControlMode({
    os: resolveGatewayControlOs(),
    localOpenclawAvailable: runtime.localOpenclawAvailable,
  });
  return resolveGatewayControlCommands(mode);
};

interface GatewayStoreState {
  status: GatewayStatus;
  refreshing: boolean;
  pendingAction: GatewayPendingAction;
  refresh: (options?: GatewayRefreshOptions) => Promise<void>;
  restart: () => Promise<void>;
  start: () => Promise<void>;
  stop: () => Promise<void>;
}

export const useGatewayStore = create<GatewayStoreState>((set, get) => ({
  status: {
    state: "stopped",
    pid: null,
    url: "http://127.0.0.1:18789",
    message: null
  },
  refreshing: false,
  pendingAction: null,
  refresh: async ({ silent = false, ...statusOptions }: GatewayRefreshOptions = {}) => {
    if (!silent) {
      set({ refreshing: true });
    }

    try {
      const status = await getGatewayStatus(statusOptions);
      set((state) => ({
        status,
        refreshing: silent ? state.refreshing : false,
      }));
    } finally {
      if (!silent) {
        set({ refreshing: false });
      }
    }
  },
  restart: async () => {
    if (get().pendingAction) return;
    set({ pendingAction: "restart" });
    try {
      const commands = await resolveGatewayControlCommandNames();
      await invokeSafe<string>(commands.restart);
      const reachable = await invokeSafe<boolean>("health_check_gateway", undefined, false);
      const status = await getGatewayStatus();
      set({ status: { ...status, state: reachable ? "running" : status.state } });
    } finally {
      set({ pendingAction: null });
    }
  },
  start: async () => {
    if (get().pendingAction) return;
    set({ pendingAction: "start" });
    try {
      const commands = await resolveGatewayControlCommandNames();
      await invokeSafe(commands.start);
      const status = await getGatewayStatus();
      set({ status });
    } finally {
      set({ pendingAction: null });
    }
  },
  stop: async () => {
    if (get().pendingAction) return;
    set({ pendingAction: "stop" });
    try {
      const commands = await resolveGatewayControlCommandNames();
      await invokeSafe(commands.stop);
      const status = await getGatewayStatus();
      set({ status });
    } finally {
      set({ pendingAction: null });
    }
  }
}));
