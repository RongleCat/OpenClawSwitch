import { useEffect, useState } from "react";
import { Plus, Save, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { loadConfigDocument, saveConfigDocument } from "@/lib/config";
import type { OpenClawConfig } from "@/types/config";

interface ProviderDraft {
  name: string;
  baseUrl: string;
  apiKey: string;
}

export function ModelsPage() {
  const [config, setConfig] = useState<OpenClawConfig | null>(null);
  const [path, setPath] = useState("");
  const [drafts, setDrafts] = useState<ProviderDraft[]>([]);

  useEffect(() => {
    void loadConfigDocument().then(([nextConfig, info]) => {
      setConfig(nextConfig);
      setPath(info.path);
      const providers = Object.entries(nextConfig.models?.providers ?? {}).map(([name, provider]) => ({
        name,
        baseUrl: provider.baseUrl,
        apiKey: provider.apiKey ?? ""
      }));
      setDrafts(providers.length > 0 ? providers : [{ name: "openai", baseUrl: "https://api.openai.com/v1", apiKey: "" }]);
    }).catch(() => {
      setDrafts([{ name: "openai", baseUrl: "https://api.openai.com/v1", apiKey: "" }]);
    });
  }, []);

  function updateDraft(index: number, patch: Partial<ProviderDraft>) {
    setDrafts((current) => current.map((draft, draftIndex) => (draftIndex === index ? { ...draft, ...patch } : draft)));
  }

  async function handleSave() {
    if (!config || !path) return;
    const providers = Object.fromEntries(
      drafts.filter((draft) => draft.name.trim()).map((draft) => [
        draft.name,
        {
          baseUrl: draft.baseUrl,
          apiKey: draft.apiKey || undefined,
          models: config.models?.providers?.[draft.name]?.models ?? []
        }
      ]),
    );
    const nextConfig: OpenClawConfig = {
      ...config,
      models: {
        ...(config.models ?? {}),
        providers
      }
    };
    await saveConfigDocument(nextConfig, path);
    setConfig(nextConfig);
    toast.success("模型配置已保存");
  }

  return (
    <div className="space-y-3">
      <Card className="bg-card/60 backdrop-blur-md">
        <CardContent className="pt-4">
          <div className="space-y-3">
            {drafts.map((draft, index) => (
              <div key={`${draft.name}-${index}`} className="flex items-center gap-2">
                <Input className="flex-1" value={draft.name} onChange={(event) => updateDraft(index, { name: event.target.value })} placeholder="名称" />
                <Input className="flex-[2]" value={draft.baseUrl} onChange={(event) => updateDraft(index, { baseUrl: event.target.value })} placeholder="baseUrl" />
                <Input className="flex-1" value={draft.apiKey} onChange={(event) => updateDraft(index, { apiKey: event.target.value })} placeholder="apiKey" />
                <Button variant="ghost" size="icon" className="h-9 w-9" onClick={() => setDrafts((current) => current.filter((_, currentIndex) => currentIndex !== index))}>
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            ))}
            <div className="flex gap-2 pt-2">
              <Button variant="outline" onClick={() => setDrafts((current) => [...current, { name: "", baseUrl: "", apiKey: "" }])}>
                <Plus className="mr-2 h-4 w-4" />
                添加
              </Button>
              <Button onClick={() => void handleSave()}>
                <Save className="mr-2 h-4 w-4" />
                保存
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
