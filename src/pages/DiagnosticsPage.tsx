import { useEffect, useState } from "react";
import { Bug, ExternalLink, Play, RefreshCw, Square } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useGatewayStore } from "@/stores/gatewayStore";
import { invokeSafe } from "@/lib/desktop";

export function DiagnosticsPage() {
  const { status, refresh } = useGatewayStore();
  const [doctorRunning, setDoctorRunning] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function runDoctor(fix: boolean) {
    setDoctorRunning(true);
    await invokeSafe("start_openclaw_doctor", { fix }, true);
    setLogs((current) => [`已触发 openclaw doctor${fix ? " --fix" : ""}`, ...current]);
    toast.success(`已启动诊断${fix ? "与修复" : ""}`);
    setDoctorRunning(false);
  }

  return (
    <div className="grid gap-5 xl:grid-cols-[0.9fr_1.4fr]">
      <Card>
        <CardHeader>
          <CardTitle>诊断动作</CardTitle>
          <CardDescription>把最常用的日志、doctor 和 WebUI 操作集中在一个面板。</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3">
          <Button onClick={() => void invokeSafe("start_openclaw_logs_follow").then(() => setLogs((current) => ["开始跟踪 openclaw logs --follow", ...current]))}>
            <Play className="mr-2 h-4 w-4" />
            开始跟踪日志
          </Button>
          <Button variant="outline" disabled={doctorRunning} onClick={() => void runDoctor(false)}>
            <Bug className="mr-2 h-4 w-4" />
            运行 doctor
          </Button>
          <Button variant="outline" disabled={doctorRunning} onClick={() => void runDoctor(true)}>
            <RefreshCw className="mr-2 h-4 w-4" />
            运行 doctor --fix
          </Button>
          <Button variant="secondary" onClick={() => void invokeSafe("open_web_ui").then(() => toast.success("已打开 OpenClaw WebUI"))}>
            OpenClaw WebUI
            <ExternalLink className="ml-2 h-4 w-4" />
          </Button>
          <Button variant="ghost" onClick={() => void invokeSafe("stop_gateway").then(() => toast.success("已发送停止命令"))}>
            <Square className="mr-2 h-4 w-4" />
            停止网关
          </Button>
          <div className="rounded-[1.4rem] border border-border/60 bg-background/70 px-4 py-3 text-sm text-muted-foreground">当前网关状态：{status.state}</div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>运行日志</CardTitle>
          <CardDescription>后续会继续接入实时事件流，这里先保留新的容器布局与滚动行为。</CardDescription>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-[420px] rounded-[1.6rem] border border-border/60 bg-[linear-gradient(180deg,rgba(42,52,30,0.04),transparent)]">
            <div className="space-y-3 p-4">
              {logs.length > 0 ? logs.map((line, index) => (
                <div key={`${line}-${index}`} className="rounded-[1.2rem] border border-border/60 bg-background/80 px-4 py-3 text-sm text-foreground">
                  {line}
                </div>
              )) : (
                <div className="rounded-[1.2rem] border border-dashed border-border/70 px-4 py-8 text-center text-sm text-muted-foreground">
                  还没有新的诊断日志。可以先运行一次 doctor 或打开 WebUI。
                </div>
              )}
            </div>
          </ScrollArea>
        </CardContent>
      </Card>
    </div>
  );
}
