# UI 组件系统

## 1. 设计系统概述

### 1.1 设计理念

OpenClawSwitch 采用现代化设计系统，基于以下原则：

- **简洁高效**：减少视觉干扰，聚焦核心功能
- **一致性**：统一的组件语言和交互模式
- **可访问性**：遵循 WCAG 2.1 标准
- **响应式**：适配不同屏幕尺寸

### 1.2 技术栈

| 技术 | 用途 | 版本 |
|------|------|------|
| shadcn/ui | 组件库基础 | latest |
| Radix Primitives | 无头组件 | latest |
| Tailwind CSS | 样式系统 | 3.4.x |
| class-variance-authority | 变体管理 | 0.7.x |
| lucide-react | 图标库 | 0.469.x |

---

## 2. 设计令牌 (Design Tokens)

### 2.1 颜色系统

采用 HSL 色彩空间，通过 CSS 变量实现主题切换：

```css
/* 基础颜色 */
--background: 色相 饱和度 亮度
--foreground: 色相 饱和度 亮度
--card: ...
--card-foreground: ...

/* 语义颜色 */
--primary: 100 20% 40%      /* 主色调 - 绿色系 */
--primary-foreground: ...
--secondary: ...
--secondary-foreground: ...
--destructive: 0 84% 60%    /* 危险操作 - 红色 */
--destructive-foreground: ...
--muted: ...
--muted-foreground: ...
--accent: ...
--accent-foreground: ...

/* 组件颜色 */
--border: ...
--input: ...
--ring: ...
```

**基准色值**：
- 主色调：中性色 (neutral) - 通过 `components.json` 配置
- 成功状态：绿色系
- 警告状态：黄色系
- 错误状态：红色系
- 信息状态：蓝色系

### 2.2 圆角系统

```typescript
borderRadius: {
  lg: "0.5rem",   // 8px - 大圆角
  md: "0.375rem", // 6px - 中圆角
  sm: "0.25rem",  // 4px - 小圆角
  full: "9999px"  // 胶囊形
}
```

### 2.3 动画系统

**预定义动画**：
```typescript
keyframes: {
  "accordion-down": {
    from: { height: 0 },
    to: { height: "var(--radix-accordion-content-height)" }
  },
  "accordion-up": {
    from: { height: "var(--radix-accordion-content-height)" },
    to: { height: 0 }
  }
}
```

**缓动函数**：
- `ease-out` - 减速曲线 (默认)
- `ease-in` - 加速曲线
- `ease-in-out` - 对称曲线
- `linear` - 线性

---

## 3. 基础组件

### 3.1 Button (按钮)

**文件**：`src/components/ui/button.tsx`

**变体**：

| 变体 | 用途 | 视觉效果 |
|------|------|----------|
| `default` | 主要操作 | 绿色背景，悬浮上移，阴影 |
| `secondary` | 次要操作 | 灰色背景 |
| `outline` | 边框按钮 | 透明背景，边框 |
| `ghost` | 幽灵按钮 | 透明背景，悬浮填充 |
| `destructive` | 危险操作 | 红色背景 |

**尺寸**：

| 尺寸 | 高度 | 内边距 | 字号 |
|------|------|--------|------|
| `sm` | 32px | 12px 8px | 12px |
| `default` | 40px | 16px 12px | 14px |
| `lg` | 44px | 20px 10px | 14px |
| `icon` | 40px | - | - |

**使用示例**：
```tsx
import { Button } from "@/components/ui/button";

// 主要按钮
<Button onClick={handleSubmit}>提交</Button>

// 边框按钮
<Button variant="outline">取消</Button>

// 危险操作
<Button variant="destructive">删除</Button>

// 图标按钮
<Button variant="ghost" size="icon">
  <Settings className="h-4 w-4" />
</Button>
```

**设计特点**：
- 圆角：`rounded-full` (胶囊形)
- 阴影：`shadow-[0_10px_30px_rgba(108,124,64,0.24)]`
- 悬浮效果：`hover:-translate-y-0.5`
- 过渡：`transition-all duration-200`

---

### 3.2 Card (卡片)

**文件**：`src/components/ui/card.tsx`

**组件结构**：
```tsx
<Card>
  <CardHeader>
    <CardTitle>标题</CardTitle>
    <CardDescription>描述</CardDescription>
  </CardHeader>
  <CardContent>内容</CardContent>
  <CardFooter>底部</CardFooter>
</Card>
```

**样式特点**：
- 边框：`border border-border/60`
- 背景：`bg-card/80` (半透明)
- 圆角：继承主题配置
- 间距：内部组件自动处理

**使用示例**：
```tsx
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

<Card className="col-span-2">
  <CardHeader>
    <CardTitle>网关概况</CardTitle>
    <CardDescription>实时监控网关状态</CardDescription>
  </CardHeader>
  <CardContent>
    {/* 卡片内容 */}
  </CardContent>
</Card>
```

---

### 3.3 Input (输入框)

**文件**：`src/components/ui/input.tsx`

**样式特点**：
- 边框：`border border-input`
- 背景：`bg-background`
- 圆角：`rounded-md`
- 焦点：`focus-visible:ring-2 focus-visible:ring-ring`

**使用示例**：
```tsx
import { Input } from "@/components/ui/input";

<Input
  type="email"
  placeholder="请输入邮箱"
  value={email}
  onChange={(e) => setEmail(e.target.value)}
/>
```

---

### 3.4 Badge (徽章)

**文件**：`src/components/ui/badge.tsx`

**变体**：

| 变体 | 用途 | 颜色 |
|------|------|------|
| `default` | 默认 | 主色调 |
| `secondary` | 次要 | 灰色 |
| `destructive` | 危险 | 红色 |
| `outline` | 边框 | 透明 |
| `success` | 成功 | 绿色 |
| `warning` | 警告 | 黄色 |

**使用示例**：
```tsx
import { Badge } from "@/components/ui/badge";

<Badge variant="success">运行中</Badge>
<Badge variant="warning">待修复</Badge>
<Badge variant="outline">可选</Badge>
```

---

### 3.5 Switch (开关)

**文件**：`src/components/ui/switch.tsx`

**样式特点**：
- 基于 Radix Switch
- 支持主题切换
- 支持禁用状态

**使用示例**：
```tsx
import { Switch } from "@/components/ui/switch";

<Switch
  checked={launchAtStartup}
  onCheckedChange={setLaunchAtStartup}
/>
```

---

### 3.6 ScrollArea (滚动区域)

**文件**：`src/components/ui/scroll-area.tsx`

**样式特点**：
- 自定义滚动条样式
- 支持横向和纵向滚动
- 平滑滚动动画

**使用示例**：
```tsx
import { ScrollArea } from "@/components/ui/scroll-area";

<ScrollArea className="h-[400px] w-full">
  {/* 可滚动内容 */}
</ScrollArea>
```

---

## 4. 复合组件

### 4.1 AppShell (应用壳)

**文件**：`src/components/shell/AppShell.tsx`

**结构**：
```tsx
<div className="app-shell">
  <AppHeader />           {/* 顶部导航栏 */}
  <div className="content">
    <Outlet />            {/* 路由内容 */}
  </div>
</div>
```

**功能**：
- 固定顶部导航
- 响应式布局
- 路由出口

---

### 4.2 AppHeader (顶部导航栏)

**文件**：`src/components/shell/AppHeader.tsx`

**组成**：
- 应用 Logo/标题
- 主导航菜单
- 全局搜索入口
- 窗口控制按钮

---

### 4.3 GlobalSearch (全局搜索)

**文件**：`src/components/search/GlobalSearch.tsx`

**功能**：
- 快速导航到页面
- 搜索配置项
- 键盘快捷键支持 (`Ctrl/Cmd + K`)

---

## 5. 图标系统

### 5.1 图标库

使用 `lucide-react` 作为主图标库：

```tsx
import {
  Activity,      // 活动/状态
  Bot,           // AI/机器人
  FolderTree,    // 目录/文件
  Globe,         // 网络/全局
  Logs,          // 日志
  PlugZap,       // 插件/扩展
  Settings,      // 设置
  Home,          // 首页
  Search         // 搜索
} from "lucide-react";
```

### 5.2 图标使用规范

```tsx
// 标准尺寸 (16x16)
<Icon className="h-4 w-4" />

// 大尺寸 (20x20)
<Icon className="h-5 w-5" />

// 配合文本
<div className="flex items-center gap-2">
  <Icon className="h-4 w-4" />
  <span>文本</span>
</div>
```

---

## 6. 响应式设计

### 6.1 断点系统

```typescript
screens: {
  "sm": "640px",   // 手机横屏
  "md": "768px",   // 平板
  "lg": "1024px",  // 小屏桌面
  "xl": "1280px",  // 标准桌面
  "2xl": "1400px"  // 大屏桌面
}
```

### 6.2 布局模式

**页面网格**：
```css
.page-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1.5rem;
}

@media (min-width: 768px) {
  .page-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (min-width: 1024px) {
  .page-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}
```

---

## 7. 辅助工具

### 7.1 cn() 工具函数

**文件**：`src/lib/utils.ts`

```typescript
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

**用途**：
- 合并多个类名
- 自动处理 Tailwind 冲突
- 支持条件类名

**使用示例**：
```tsx
function Card({ className, ...props }: CardProps) {
  return (
    <div
      className={cn(
        "rounded-lg border bg-card",
        "shadow-sm",
        className // 用户传入的额外类名
      )}
      {...props}
    />
  );
}
```

### 7.2 class-variance-authority

用于管理组件变体：

```tsx
import { cva } from "class-variance-authority";

const buttonVariants = cva(
  "base-styles",
  {
    variants: {
      variant: {
        default: "default-styles",
        outline: "outline-styles"
      },
      size: {
        sm: "sm-styles",
        lg: "lg-styles"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "default"
    }
  }
);
```

---

## 8. 暗色主题

### 8.1 主题切换机制

通过 `darkMode: ["class"]` 配置实现：

```tsx
// HTML 标签添加 dark 类
<html class="dark">

// Tailwind 自动应用 dark: 变体
<div className="bg-white dark:bg-gray-900" />
```

### 8.2 CSS 变量主题

```css
/* 亮色主题 */
:root {
  --background: 0 0% 100%;
  --foreground: 222 47% 11%;
  /* ... */
}

/* 暗色主题 */
.dark {
  --background: 222 47% 11%;
  --foreground: 210 40% 98%;
  /* ... */
}
```

---

## 9. 组件使用最佳实践

### 9.1 组合优于继承

```tsx
// ✅ 推荐：使用组合
<Card>
  <CardHeader>
    <CardTitle>标题</CardTitle>
  </CardHeader>
  <CardContent>内容</CardContent>
</Card>

// ❌ 不推荐：自定义样式覆盖
<div className="custom-card-styles">
  <div className="custom-header">标题</div>
  <div className="custom-content">内容</div>
</div>
```

### 9.2 使用 TypeScript 类型

```tsx
// ✅ 推荐：完整类型定义
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement>,
  VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

// ❌ 不推荐：any 类型
function Button(props: any) { ... }
```

### 9.3 可访问性优先

```tsx
// ✅ 推荐：语义化 + ARIA
<Button aria-label="关闭对话框">
  <X className="h-4 w-4" />
</Button>

// ❌ 不推荐：仅图标无语义
<div onClick={close}>
  <X className="h-4 w-4" />
</div>
```

---

*最后更新：2026-03-19*
