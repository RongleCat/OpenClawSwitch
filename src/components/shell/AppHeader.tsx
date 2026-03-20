import { Loader2, Power, RefreshCw } from "lucide-react";
import { useLocation } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useGatewayStore } from "@/stores/gatewayStore";

const titles: Record<string, string> = {
  "/": "概览",
  "/models": "模型",
  "/channels": "渠道",
  "/settings": "设置",
};

export function AppHeader() {
  const location = useLocation();
  const { status, refreshing, refresh, restart } = useGatewayStore();
  const title = titles[location.pathname] ?? titles["/"];

  return (
    <header className="flex min-w-0 items-center justify-between border-b border-border/60 bg-background/60 px-5 py-3 backdrop-blur-md">
      <h1 className="font-display text-lg font-semibold text-foreground">{title}</h1>
      <Card className="flex items-center gap-2 rounded-2xl bg-card/70 px-3 py-2 backdrop-blur-md">
        <Badge
          variant={
            status.state === "running" ? "success" : status.state === "error" ? "destructive" : "warning"
          }
          className="rounded-full px-2.5 py-1 text-xs"
        >
          {status.state}
        </Badge>
        <span className="text-xs text-muted-foreground">{status.url ?? "http://127.0.0.1:18789"}</span>
        <div className="mx-2 h-4 w-px bg-border" />
        <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => void refresh()}>
          {refreshing ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
        </Button>
        <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => void restart()}>
          <Power className="h-3.5 w-3.5" />
        </Button>
      </Card>
    </header>
  );
}
