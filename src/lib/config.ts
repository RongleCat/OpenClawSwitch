import type { OpenClawConfig } from "@/types/config";
import { invokeSafe } from "@/lib/desktop";

export interface ConfigFileInfo {
  path: string;
  mode: "local" | "remote" | "ssh";
  fileName: string;
  dirPath: string;
}

export async function loadConfigDocument() {
  return invokeSafe<[OpenClawConfig, ConfigFileInfo]>("load_default_config");
}

export async function saveConfigDocument(config: OpenClawConfig, path: string) {
  return invokeSafe<void>("save_config", { config, path });
}

export async function ensureDefaultConfig() {
  return invokeSafe<string>("generate_default_config");
}
