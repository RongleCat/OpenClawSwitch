import { useState } from "react";
import { FolderOpen, RotateCcw } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { useSettingsStore } from "@/stores/settingsStore";
import { invokeSafe } from "@/lib/desktop";

export function SettingsPage() {
  const launchAtStartup = useSettingsStore((state) => state.launchAtStartup);
  const setLaunchAtStartup = useSettingsStore((state) => state.setLaunchAtStartup);
  const setSetupComplete = useSettingsStore((state) => state.setSetupComplete);
  const [busy, setBusy] = useState(false);

  return (
    <div className="space-y-3">
      <Card className="bg-card/60 backdrop-blur-md">
        <CardContent className="pt-4">
          <div className="space-y-3">
            <div className="flex items-center justify-between rounded-xl border border-border/60 bg-background/50 px-4 py-3">
              <div>
                <div className="text-sm font-medium">开机自启</div>
                <p className="text-xs text-muted-foreground">应用启动后自动拉起网关</p>
              </div>
              <Switch checked={launchAtStartup} onCheckedChange={(value) => void setLaunchAtStartup(value)} />
            </div>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={() => void invokeSafe("open_path_in_default_app", { path: "~/.openclaw" }, undefined).catch(() => toast.info("请手动打开 ~/.openclaw"))}>
                <FolderOpen className="mr-2 h-3.5 w-3.5" />
                配置目录
              </Button>
              <Button variant="ghost" size="sm" disabled={busy} onClick={() => { setBusy(true); void setSetupComplete(false).then(() => toast.success("下次启动会重新进入引导")).finally(() => setBusy(false)); }}>
                <RotateCcw className="mr-2 h-3.5 w-3.5" />
                重置引导
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
