import { create } from "zustand";
import { invokeSafe } from "@/lib/desktop";
import { getGatewayStatus, type GatewayStatus } from "@/lib/runtime";

interface GatewayStoreState {
  status: GatewayStatus;
  refreshing: boolean;
  refresh: () => Promise<void>;
  restart: () => Promise<void>;
  start: () => Promise<void>;
  stop: () => Promise<void>;
}

export const useGatewayStore = create<GatewayStoreState>((set) => ({
  status: {
    state: "stopped",
    pid: null,
    url: "http://127.0.0.1:18789",
    message: null
  },
  refreshing: false,
  refresh: async () => {
    set({ refreshing: true });
    const status = await getGatewayStatus();
    set({ status, refreshing: false });
  },
  restart: async () => {
    await invokeSafe<string>("restart_gateway");
    const reachable = await invokeSafe<boolean>("health_check_gateway", undefined, false);
    const status = await getGatewayStatus();
    set({ status: { ...status, state: reachable ? "running" : status.state } });
  },
  start: async () => {
    await invokeSafe("start_gateway");
    const status = await getGatewayStatus();
    set({ status });
  },
  stop: async () => {
    await invokeSafe("stop_gateway");
    const status = await getGatewayStatus();
    set({ status });
  }
}));
