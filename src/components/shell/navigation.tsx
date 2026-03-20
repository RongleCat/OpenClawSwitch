import { Bot, GalleryVerticalEnd, RadioTower, Settings2 } from "lucide-react";

export const navigationItems = [
  { to: "/", label: "概览", icon: GalleryVerticalEnd },
  { to: "/models", label: "模型", icon: Bot },
  { to: "/channels", label: "渠道", icon: RadioTower },
  { to: "/settings", label: "设置", icon: Settings2 }
];
