import { startTransition, useEffect, useState } from "react";
import { Activity, Box, FolderTree, Globe, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { scheduleDeferredTask } from "@/domain/deferredTask";
import { resolveGatewayQuickActionLabel, resolveGatewayQuickActionState } from "@/domain/gatewayServiceAction";
import { getRuntimeHealth } from "@/lib/runtime";
import { useGatewayStore } from "@/stores/gatewayStore";

export function OverviewPage() {
  const { status, refresh, start, stop, restart, pendingAction } = useGatewayStore();
  const [runtime, setRuntime] = useState({
    nodeReady: true,
    openclawReady: true,
    configReady: false,
    dataDir: ""
  });

  useEffect(() => {
    let systemRefreshTimeoutId: number | null = null;

    const dispose = scheduleDeferredTask(() => {
      void refresh({ includeSystemStatus: false, silent: true });
      void getRuntimeHealth().then((nextRuntime) => {
        startTransition(() => {
          setRuntime(nextRuntime);
        });
      });

      systemRefreshTimeoutId = window.setTimeout(() => {
        void refresh({ silent: true });
      }, 400);
    });

    return () => {
      dispose();
      if (systemRefreshTimeoutId !== null) {
        window.clearTimeout(systemRefreshTimeoutId);
      }
    };
  }, [refresh]);

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-3">
        <StatusCard status={status.state} nodeReady={runtime.nodeReady} />
        <QuickActions
          onStart={() => void start()}
          onStop={() => void stop()}
          onRestart={() => void restart()}
          running={status.state === "running"}
          pendingAction={pendingAction}
        />
      </div>
      <RuntimeHealth openclawReady={runtime.openclawReady} configReady={runtime.configReady} />
    </div>
  );
}

function StatusCard({ status, nodeReady }: { status: string; nodeReady: boolean }) {
  return (
    <Card className="bg-card/60 backdrop-blur-md">
      <CardContent className="pt-4">
        <div className="flex items-center justify-between">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <Activity className="h-4 w-4 text-muted-foreground" />
              <span className="text-xs uppercase tracking-[0.2em] text-muted-foreground">网关状态</span>
            </div>
            <Badge variant={status === "running" ? "success" : status === "error" ? "destructive" : "warning"} className="rounded-full">
              {status}
            </Badge>
          </div>
          <div className="text-right">
            <div className="flex items-center gap-2">
              <Box className="h-4 w-4 text-muted-foreground" />
              <span className="text-xs text-muted-foreground">Node</span>
            </div>
            <span className="text-sm font-medium">{nodeReady ? "就绪" : "异常"}</span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function QuickActions({
  onStart,
  onStop,
  onRestart,
  running,
  pendingAction,
}: {
  onStart: () => void;
  onStop: () => void;
  onRestart: () => void;
  running: boolean;
  pendingAction: "start" | "restart" | "stop" | null;
}) {
  const startState = resolveGatewayQuickActionState({
    actionId: "start",
    baseDisabled: running,
    pendingActionId: pendingAction,
  });
  const restartState = resolveGatewayQuickActionState({
    actionId: "restart",
    baseDisabled: !running,
    pendingActionId: pendingAction,
  });
  const stopState = resolveGatewayQuickActionState({
    actionId: "stop",
    baseDisabled: !running,
    pendingActionId: pendingAction,
  });

  return (
    <Card className="bg-card/60 backdrop-blur-md">
      <CardContent className="pt-4">
        <div className="flex gap-2">
          <Button size="sm" className="flex-1 gap-2" onClick={onStart} disabled={startState.disabled} aria-busy={startState.loading}>
            {startState.loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
            {resolveGatewayQuickActionLabel("start", "启动", startState.loading)}
          </Button>
          <Button size="sm" variant="outline" className="flex-1 gap-2" onClick={onRestart} disabled={restartState.disabled} aria-busy={restartState.loading}>
            {restartState.loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
            {resolveGatewayQuickActionLabel("restart", "重启", restartState.loading)}
          </Button>
          <Button size="sm" variant="outline" className="flex-1 gap-2" onClick={onStop} disabled={stopState.disabled} aria-busy={stopState.loading}>
            {stopState.loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
            {resolveGatewayQuickActionLabel("stop", "停止", stopState.loading)}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function RuntimeHealth({ openclawReady, configReady }: { openclawReady: boolean; configReady: boolean }) {
  return (
    <Card className="bg-card/60 backdrop-blur-md">
      <CardContent className="pt-4">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <FolderTree className="h-4 w-4 text-muted-foreground" />
            <span className="text-xs text-muted-foreground">配置</span>
          </div>
          <Badge variant={configReady ? "success" : "warning"} className="rounded-full">{configReady ? "就绪" : "未配置"}</Badge>
          <div className="ml-4 h-4 w-px bg-border" />
          <div className="flex items-center gap-2">
            <Globe className="h-4 w-4 text-muted-foreground" />
            <span className="text-xs text-muted-foreground">OpenClaw</span>
          </div>
          <Badge variant={openclawReady ? "success" : "warning"} className="rounded-full">{openclawReady ? "就绪" : "异常"}</Badge>
        </div>
      </CardContent>
    </Card>
  );
}
