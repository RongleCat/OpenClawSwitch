import { useEffect, useState } from "react";
import { create } from "zustand";
import { invokeSafe, tryInvoke } from "@/lib/desktop";
import { resolveSetupCompletion } from "@/domain/setupBootstrap";
import { getRuntimeHealth } from "@/lib/runtime";

export interface DesktopPreferences {
  launchAtStartup: boolean;
  setupComplete: boolean;
}

interface SettingsState extends DesktopPreferences {
  loading: boolean;
  initialize: () => Promise<void>;
  setLaunchAtStartup: (enabled: boolean) => Promise<void>;
  setSetupComplete: (complete: boolean) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  launchAtStartup: false,
  setupComplete: false,
  loading: true,
  initialize: async () => {
    const prefs = await tryInvoke<DesktopPreferences>("get_desktop_preferences");
    const launchAtStartup = await invokeSafe<boolean>("get_launch_at_startup_enabled", undefined, prefs?.launchAtStartup ?? false);
    const runtime = await getRuntimeHealth();
    set({
      launchAtStartup,
      setupComplete: resolveSetupCompletion(prefs?.setupComplete ?? false, runtime.shouldSkipSetup),
      loading: false
    });
  },
  setLaunchAtStartup: async (enabled) => {
    await invokeSafe("set_launch_at_startup_enabled", { enabled }, undefined);
    const current = await invokeSafe<DesktopPreferences>("set_desktop_preferences", {
      preferences: { launchAtStartup: enabled }
    }, {
      launchAtStartup: enabled,
      setupComplete: get().setupComplete
    });
    set({ launchAtStartup: current.launchAtStartup });
  },
  setSetupComplete: async (complete) => {
    const current = await invokeSafe<DesktopPreferences>("set_desktop_preferences", {
      preferences: { setupComplete: complete }
    }, {
      launchAtStartup: get().launchAtStartup,
      setupComplete: complete
    });
    set({ setupComplete: current.setupComplete });
  }
}));

export function useSettingsBootstrap() {
  const [loading, setLoading] = useState(true);
  const setupComplete = useSettingsStore((state) => state.setupComplete);
  const initialize = useSettingsStore((state) => state.initialize);

  useEffect(() => {
    initialize().finally(() => setLoading(false));
  }, [initialize]);

  return { loading, setupComplete };
}
