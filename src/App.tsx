import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Navigate, Route, Routes, useLocation, useNavigate } from "react-router-dom";
import { AppShell } from "@/components/shell/AppShell";
import { ChannelsPage } from "@/pages/ChannelsPage";
import { ModelsPage } from "@/pages/ModelsPage";
import { OverviewPage } from "@/pages/OverviewPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { SetupPage } from "@/pages/SetupPage";
import { resolveSetupRedirect } from "@/domain/setupBootstrap";
import { useSettingsBootstrap } from "@/stores/settingsStore";

export default function App() {
  const { loading, setupComplete } = useSettingsBootstrap();
  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    void listen<string>("desktop:navigate", (event) => {
      navigate(event.payload);
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      unlisten?.();
    };
  }, [navigate]);

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center bg-background text-foreground">
        <div className="rounded-2xl border border-border/60 bg-card/80 px-5 py-4 shadow-lg backdrop-blur-md">
          正在加载...
        </div>
      </div>
    );
  }

  const redirect = resolveSetupRedirect(location.pathname, setupComplete);
  if (redirect) {
    return <Navigate to={redirect} replace />;
  }

  return (
    <Routes>
      <Route path="/setup" element={<SetupPage />} />
      <Route element={<AppShell />}>
        <Route index element={<OverviewPage />} />
        <Route path="/models" element={<ModelsPage />} />
        <Route path="/channels" element={<ChannelsPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Route>
      <Route path="*" element={<Navigate to={setupComplete ? "/" : "/setup"} replace />} />
    </Routes>
  );
}
