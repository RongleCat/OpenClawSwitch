import { Sparkles } from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";
import { ScrollArea } from "@/components/ui/scroll-area";
import { AppHeader } from "@/components/shell/AppHeader";
import { navigationItems } from "@/components/shell/navigation";
import { cn } from "@/lib/utils";

export function AppShell() {
  return (
    <div className="shell-root">
      <aside className="shell-sidebar">
        <div className="mb-4 flex items-center gap-2.5">
          <div className="grid h-9 w-9 place-items-center rounded-2xl bg-primary/90 text-primary-foreground shadow-lg backdrop-blur-sm">
            <Sparkles className="h-4 w-4" />
          </div>
          <div className="min-w-0">
            <div className="truncate font-display text-base font-semibold text-foreground">OpenClaw</div>
            <div className="text-[10px] uppercase tracking-[0.2em] text-muted-foreground">Console</div>
          </div>
        </div>
        <nav className="space-y-1">
          {navigationItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-2.5 rounded-xl px-3 py-2 text-sm font-medium text-muted-foreground transition-all hover:bg-accent/60 hover:text-accent-foreground hover:backdrop-blur-sm",
                  isActive && "bg-primary/90 text-primary-foreground shadow-md backdrop-blur-sm",
                )
              }
            >
              <item.icon className="h-4 w-4 flex-shrink-0" />
              <span className="truncate">{item.label}</span>
            </NavLink>
          ))}
        </nav>
      </aside>
      <div className="shell-main">
        <AppHeader />
        <ScrollArea className="min-h-0 flex-1">
          <div className="page-scroll">
            <Outlet />
          </div>
        </ScrollArea>
      </div>
    </div>
  );
}
