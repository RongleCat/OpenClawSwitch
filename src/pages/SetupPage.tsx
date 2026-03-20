import { startTransition, useEffect, useMemo, useState } from "react";
import { CheckCircle2, ChevronLeft, ChevronRight, MessageSquare, PackageCheck, Rocket, Sparkles } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { ensureDefaultConfig, loadConfigDocument, saveConfigDocument } from "@/lib/config";
import { invokeSafe } from "@/lib/desktop";
import { getRuntimeHealth } from "@/lib/runtime";
import { buildSetupConfig } from "@/lib/setupConfig";
import { useSettingsStore } from "@/stores/settingsStore";

const STEPS = [
  { id: 0, title: "欢迎", icon: Sparkles },
  { id: 1, title: "基础配置", icon: PackageCheck },
  { id: 2, title: "渠道配置", icon: MessageSquare },
  { id: 3, title: "完成", icon: Rocket }
] as const;

interface ChannelExtensionStatus {
  feishuInstalled: boolean;
  wecomInstalled: boolean;
  qqInstalled: boolean;
  dingtalkInstalled: boolean;
}

export function SetupPage() {
  const [step, setStep] = useState(0);
  const [runtimeReady, setRuntimeReady] = useState(false);
  const [providerName, setProviderName] = useState("openai");
  const [baseUrl, setBaseUrl] = useState("https://api.openai.com/v1");
  const [apiKey, setApiKey] = useState("");
  const [primaryModel, setPrimaryModel] = useState("openai/gpt-4o-mini");

  // 渠道配置状态
  const [channelStatus, setChannelStatus] = useState<ChannelExtensionStatus | null>(null);
  const [selectedChannel, setSelectedChannel] = useState<"feishu" | "dingtalk" | "">("");
  const [feishuAppId, setFeishuAppId] = useState("");
  const [feishuSecret, setFeishuSecret] = useState("");
  const [dingtalkClientId, setDingtalkClientId] = useState("");
  const [dingtalkSecret, setDingtalkSecret] = useState("");

  const setSetupComplete = useSettingsStore((state) => state.setSetupComplete);
  const navigate = useNavigate();

  useEffect(() => {
    if (step !== 0) return;
    void getRuntimeHealth().then((health) => setRuntimeReady(health.nodeReady && health.openclawReady));
  }, [step]);

  useEffect(() => {
    if (step !== 2) return;
    void invokeSafe<ChannelExtensionStatus>("get_channel_extension_status", undefined, {
      feishuInstalled: false,
      wecomInstalled: false,
      qqInstalled: false,
      dingtalkInstalled: false
    }).then(setChannelStatus);
  }, [step]);

  const canProceed = useMemo(() => {
    if (step === 0) return runtimeReady;
    if (step === 1) return Boolean(providerName && baseUrl && primaryModel);
    if (step === 2) return true; // 渠道配置可选
    return true;
  }, [baseUrl, primaryModel, providerName, runtimeReady, step]);

  async function handleProviderSetup() {
    await ensureDefaultConfig();
    const [config, info] = await loadConfigDocument();
    const nextConfig = buildSetupConfig(config, {
      providerName,
      baseUrl,
      apiKey,
      primaryModel
    });
    await saveConfigDocument(nextConfig, info.path);
  }

  async function handleChannelSetup() {
    if (selectedChannel === "feishu" && feishuAppId && feishuSecret) {
      await invokeSafe("set_feishu_channel_config", { appId: feishuAppId, appSecret: feishuSecret, enabled: true });
    } else if (selectedChannel === "dingtalk" && dingtalkClientId && dingtalkSecret) {
      await invokeSafe("set_dingtalk_channel_config", { clientId: dingtalkClientId, clientSecret: dingtalkSecret, enabled: true });
    }
  }

  async function handleNext() {
    if (step === 1) {
      await handleProviderSetup();
    }
    if (step === 2) {
      await handleChannelSetup();
      if (selectedChannel && channelStatus) {
        const channelId = selectedChannel === "feishu" ? "feishu" : "dingtalk";
        await invokeSafe("install_channel_extension", { channelId });
      }
    }
    if (step === 3) {
      await setSetupComplete(true);
      toast.success("OpenClaw 已准备就绪");
      navigate("/");
      return;
    }
    startTransition(() => setStep((value) => value + 1));
  }

  function handlePrev() {
    if (step > 0) {
      startTransition(() => setStep((value) => value - 1));
    }
  }

  return (
    <div className="setup-page">
      {/* Header: Segmented Steps */}
      <header className="setup-header">
        <div className="setup-segment">
          {STEPS.map((item, index) => (
            <button
              key={item.id}
              type="button"
              onClick={() => index <= step || (index > step && canProceed) ? setStep(index) : undefined}
              className={`setup-segment-item ${step === index ? "active" : ""} ${index < step ? "completed" : ""}`}
              disabled={index > step + 1 || (index > 0 && !canProceed)}
            >
              <item.icon className="h-4 w-4" />
              <span>{item.title}</span>
              {index < step && <CheckCircle2 className="h-3 w-3 text-emerald-600" />}
            </button>
          ))}
        </div>
      </header>

      {/* Content Area */}
      <main className="setup-content">
        <Card className="h-full flex flex-col">
          <CardHeader className="flex-shrink-0">
            <CardTitle>
              {step === 0 ? "欢迎使用新的 OpenClaw 控制台" :
               step === 1 ? "生成首份配置" :
               step === 2 ? "配置通信渠道" : "准备完成"}
            </CardTitle>
            <CardDescription>
              {step === 0 ? "Windows 和 macOS 都会直接使用应用包内的 Node 与 OpenClaw。" :
               step === 1 ? "沿用原有 Provider 与模型配置语义，只是换成了新的紧凑界面。" :
               step === 2 ? "选择一个消息渠道并配置，让 OpenClaw 能够接收和发送消息。" :
               "配置已经写入，现在可以进入主界面继续管理模型、插件和诊断。"}
            </CardDescription>
          </CardHeader>
          <CardContent className="flex-1 min-h-0 overflow-auto space-y-5">
            {/* Step 0: 欢迎 + 运行时检查 */}
            {step === 0 && (
              <>
                <div className="grid gap-4 md:grid-cols-3">
                  {[
                    ["内置完整 OpenClaw", "不再安装全局 npm 包，也不再修改 PATH。"],
                    ["关闭即驻留", "窗口关闭后驻留在托盘或状态栏，网关仍由应用托管。"],
                    ["配置仍在 ~/.openclaw", "你原来的模型配置、插件安装目录和日志路径都继续可用。"]
                  ].map(([title, description]) => (
                    <Card key={title}>
                      <CardContent className="p-5">
                        <div className="text-sm font-medium">{title}</div>
                        <p className="mt-2 text-sm text-muted-foreground">{description}</p>
                      </CardContent>
                    </Card>
                  ))}
                </div>
                <div className="space-y-3 rounded-[1.8rem] border border-border/70 bg-background/60 p-5">
                  <div className="flex items-center gap-3 text-sm">
                    <CheckCircle2 className={`h-5 w-5 ${runtimeReady ? "text-emerald-600" : "text-amber-600"}`} />
                    {runtimeReady ? "内置 Node 与 OpenClaw 已就绪。" : "正在校验运行时状态..."}
                  </div>
                  <p className="text-sm text-muted-foreground">如果配置目录缺失，后续步骤会自动生成默认 `openclaw.json`。</p>
                </div>
              </>
            )}

            {/* Step 1: 基础配置 */}
            {step === 1 && (
              <div className="grid gap-4 md:grid-cols-2">
                <Input value={providerName} onChange={(event) => setProviderName(event.target.value)} placeholder="Provider 名称" />
                <Input value={baseUrl} onChange={(event) => setBaseUrl(event.target.value)} placeholder="Base URL" />
                <Input value={apiKey} onChange={(event) => setApiKey(event.target.value)} placeholder="API Key" type="password" />
                <Input value={primaryModel} onChange={(event) => setPrimaryModel(event.target.value)} placeholder="主模型，例如 openai/gpt-4o-mini" />
              </div>
            )}

            {/* Step 2: 渠道配置 */}
            {step === 2 && (
              <div className="space-y-4">
                <div className="flex flex-wrap gap-2">
                  <Button
                    variant={selectedChannel === "feishu" ? "default" : "outline"}
                    size="sm"
                    onClick={() => setSelectedChannel("feishu")}
                  >
                    飞书
                  </Button>
                  <Button
                    variant={selectedChannel === "dingtalk" ? "default" : "outline"}
                    size="sm"
                    onClick={() => setSelectedChannel("dingtalk")}
                  >
                    钉钉
                  </Button>
                  <Button
                    variant={selectedChannel === "" ? "default" : "outline"}
                    size="sm"
                    onClick={() => setSelectedChannel("")}
                  >
                    跳过
                  </Button>
                </div>

                {selectedChannel === "feishu" && (
                  <Card>
                    <CardHeader className="pb-3">
                      <CardTitle className="text-base flex items-center justify-between">
                        飞书配置
                        <Badge variant={channelStatus?.feishuInstalled ? "success" : "secondary"}>
                          {channelStatus?.feishuInstalled ? "已安装" : "未安装"}
                        </Badge>
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div className="flex gap-2">
                        <Input
                          value={feishuAppId}
                          onChange={(event) => setFeishuAppId(event.target.value)}
                          placeholder="App ID"
                        />
                        <Input
                          value={feishuSecret}
                          onChange={(event) => setFeishuSecret(event.target.value)}
                          placeholder="App Secret"
                          type="password"
                        />
                      </div>
                    </CardContent>
                  </Card>
                )}

                {selectedChannel === "dingtalk" && (
                  <Card>
                    <CardHeader className="pb-3">
                      <CardTitle className="text-base flex items-center justify-between">
                        钉钉配置
                        <Badge variant={channelStatus?.dingtalkInstalled ? "success" : "secondary"}>
                          {channelStatus?.dingtalkInstalled ? "已安装" : "未安装"}
                        </Badge>
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div className="flex gap-2">
                        <Input
                          value={dingtalkClientId}
                          onChange={(event) => setDingtalkClientId(event.target.value)}
                          placeholder="Client ID"
                        />
                        <Input
                          value={dingtalkSecret}
                          onChange={(event) => setDingtalkSecret(event.target.value)}
                          placeholder="Client Secret"
                          type="password"
                        />
                      </div>
                    </CardContent>
                  </Card>
                )}

                {selectedChannel === "" && (
                  <div className="rounded-[1.8rem] border border-border/70 bg-background/60 p-5 text-sm text-muted-foreground">
                    <p>您可以稍后在"渠道"页面配置通信渠道。渠道配置为可选项。</p>
                  </div>
                )}
              </div>
            )}

            {/* Step 3: 完成 */}
            {step === 3 && (
              <div className="rounded-[1.8rem] border border-emerald-600/20 bg-emerald-600/10 p-5 text-sm text-emerald-800">
                启动确认完成。后续你可以在"模型""渠道""诊断""设置"页继续配置。
              </div>
            )}
          </CardContent>
        </Card>
      </main>

      {/* Footer: Navigation */}
      <footer className="setup-footer">
        <div className="flex justify-between items-center">
          <Button
            variant="ghost"
            onClick={handlePrev}
            disabled={step === 0}
            className="opacity-60 hover:opacity-100"
          >
            <ChevronLeft className="mr-2 h-4 w-4" />
            上一步
          </Button>
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <span>{step + 1}</span>
            <span>/</span>
            <span>{STEPS.length}</span>
          </div>
          <Button onClick={() => void handleNext()} disabled={!canProceed}>
            {step === 3 ? "进入控制台" : "下一步"}
            <ChevronRight className="ml-2 h-4 w-4" />
          </Button>
        </div>
      </footer>
    </div>
  );
}
