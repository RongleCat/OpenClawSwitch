import { useDeferredValue, useMemo, useState } from "react";
import { Search } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { Input } from "@/components/ui/input";

const searchItems = [
  { label: "全局搜索 · 概览", hint: "查看网关状态、日志与运行时健康", to: "/" },
  { label: "模型配置", hint: "编辑 Provider、主模型与回退模型", to: "/models" },
  { label: "渠道安装", hint: "安装并配置飞书、钉钉等渠道扩展", to: "/channels" },
  { label: "诊断工具", hint: "启动 doctor、查看日志、打开 OpenClaw WebUI", to: "/diagnostics" },
  { label: "设置", hint: "开机自启、首启状态、配置目录与版本信息", to: "/settings" }
];

export function GlobalSearch() {
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query);
  const navigate = useNavigate();
  const matches = useMemo(() => {
    const normalized = deferredQuery.trim().toLowerCase();
    if (!normalized) return searchItems;
    return searchItems.filter((item) => `${item.label} ${item.hint}`.toLowerCase().includes(normalized)).slice(0, 6);
  }, [deferredQuery]);

  return (
    <div className="relative min-w-0 flex-1">
      <Search className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
      <Input
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="全局搜索页面、动作与设置"
        className="h-11 rounded-[1.6rem] border-border/60 bg-background/90 pl-10"
      />
      {query ? (
        <div className="absolute left-0 right-0 top-[calc(100%+0.5rem)] z-40 overflow-hidden rounded-[1.6rem] border border-border/70 bg-popover shadow-[0_18px_54px_rgba(38,49,26,0.16)]">
          {matches.length > 0 ? (
            matches.map((item) => (
              <button
                key={item.to}
                className="flex w-full flex-col items-start gap-1 border-b border-border/50 px-4 py-3 text-left text-sm transition-colors last:border-b-0 hover:bg-accent hover:text-accent-foreground"
                onClick={() => {
                  navigate(item.to);
                  setQuery("");
                }}
                type="button"
              >
                <span className="font-medium">{item.label}</span>
                <span className="text-xs text-muted-foreground">{item.hint}</span>
              </button>
            ))
          ) : (
            <div className="px-4 py-4 text-sm text-muted-foreground">没有匹配结果，试试搜索“模型”或“诊断”。</div>
          )}
        </div>
      ) : null}
    </div>
  );
}
