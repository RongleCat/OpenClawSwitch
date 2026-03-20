import { useEffect, useState } from "react";
import { Download, Send } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { invokeSafe } from "@/lib/desktop";

interface ChannelExtensionStatus {
  feishuInstalled: boolean;
  wecomInstalled: boolean;
  qqInstalled: boolean;
  dingtalkInstalled: boolean;
}

export function ChannelsPage() {
  const [status, setStatus] = useState<ChannelExtensionStatus | null>(null);
  const [feishuAppId, setFeishuAppId] = useState("");
  const [feishuSecret, setFeishuSecret] = useState("");
  const [dingtalkClientId, setDingtalkClientId] = useState("");
  const [dingtalkSecret, setDingtalkSecret] = useState("");

  useEffect(() => {
    void invokeSafe<ChannelExtensionStatus>("get_channel_extension_status", undefined, {
      feishuInstalled: false,
      wecomInstalled: false,
      qqInstalled: false,
      dingtalkInstalled: false
    }).then(setStatus);
  }, []);

  return (
    <div className="space-y-3">
      <ChannelCard
        title="飞书"
        installed={status?.feishuInstalled ?? false}
        form={
          <>
            <div className="flex gap-2">
              <Input className="flex-1" value={feishuAppId} onChange={(event) => setFeishuAppId(event.target.value)} placeholder="App ID" />
              <Input className="flex-1" value={feishuSecret} onChange={(event) => setFeishuSecret(event.target.value)} placeholder="App Secret" />
            </div>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={() => void invokeSafe("install_channel_extension", { channelId: "feishu" }).then(() => toast.success("飞书扩展安装完成"))}>
                <Download className="mr-2 h-3.5 w-3.5" />
                安装
              </Button>
              <Button size="sm" onClick={() => void invokeSafe("set_feishu_channel_config", { appId: feishuAppId, appSecret: feishuSecret, enabled: true }).then(() => toast.success("飞书配置已写入"))}>
                <Send className="mr-2 h-3.5 w-3.5" />
                保存
              </Button>
            </div>
          </>
        }
      />
      <ChannelCard
        title="钉钉"
        installed={status?.dingtalkInstalled ?? false}
        form={
          <>
            <div className="flex gap-2">
              <Input className="flex-1" value={dingtalkClientId} onChange={(event) => setDingtalkClientId(event.target.value)} placeholder="Client ID" />
              <Input className="flex-1" value={dingtalkSecret} onChange={(event) => setDingtalkSecret(event.target.value)} placeholder="Client Secret" />
            </div>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={() => void invokeSafe("install_channel_extension", { channelId: "dingtalk" }).then(() => toast.success("钉钉扩展安装完成"))}>
                <Download className="mr-2 h-3.5 w-3.5" />
                安装
              </Button>
              <Button size="sm" onClick={() => void invokeSafe("set_dingtalk_channel_config", { clientId: dingtalkClientId, clientSecret: dingtalkSecret, enabled: true }).then(() => toast.success("钉钉配置已写入"))}>
                <Send className="mr-2 h-3.5 w-3.5" />
                保存
              </Button>
            </div>
          </>
        }
      />
    </div>
  );
}

function ChannelCard({
  title,
  installed,
  form
}: {
  title: string;
  installed: boolean;
  form: React.ReactNode;
}) {
  return (
    <Card className="bg-card/60 backdrop-blur-md">
      <CardContent className="pt-4">
        <div className="mb-3 flex items-center justify-between">
          <span className="font-medium">{title}</span>
          <Badge variant={installed ? "success" : "secondary"} className="rounded-full">{installed ? "已安装" : "未安装"}</Badge>
        </div>
        <div className="space-y-3">{form}</div>
      </CardContent>
    </Card>
  );
}
