# 开发指南

## 1. 开发环境设置

### 1.1 系统要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Node.js | 18.x | 20.x |
| Rust | 1.70 | 1.75+ |
| pnpm | 8.x | 9.x |
| Git | 2.x | 2.40+ |

### 1.2 开发工具

**必需工具**：
- [Visual Studio Code](https://code.visualstudio.com/) - 推荐编辑器
- [Rustup](https://rustup.rs/) - Rust 工具链管理
- [Node.js](https://nodejs.org/) - JavaScript 运行时

**推荐 VS Code 扩展**：
- `rust-analyzer` - Rust 语言支持
- `ESLint` - JavaScript/TypeScript  linting
- `Prettier` - 代码格式化
- `Tailwind CSS IntelliSense` - Tailwind 提示
- `Tauri` - Tauri 开发支持

### 1.3 环境安装

#### Windows

```powershell
# 1. 安装 Rust
winget install Rustlang.Rustup

# 2. 安装 Node.js
winget install OpenJS.NodeJS.LTS

# 3. 安装 pnpm
npm install -g pnpm

# 4. 安装依赖
pnpm install
```

#### macOS

```bash
# 1. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 安装 Node.js (推荐 via Homebrew)
brew install node@20

# 3. 安装 pnpm
npm install -g pnpm

# 4. 安装依赖
pnpm install
```

#### Linux

```bash
# 1. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 安装 Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# 3. 安装 pnpm
npm install -g pnpm

# 4. 安装依赖
pnpm install
```

---

## 2. 项目结构

```
openclaw-manager-tauri/
├── src/                      # 前端源码 (React + TypeScript)
│   ├── App.tsx               # 应用根组件
│   ├── main.tsx              # 入口文件
│   ├── assets/               # 静态资源
│   ├── components/           # 组件库
│   │   ├── ui/               # shadcn/ui 基础组件
│   │   ├── shell/            # 应用壳组件
│   │   └── search/           # 搜索组件
│   ├── pages/                # 页面组件
│   ├── stores/               # Zustand 状态管理
│   ├── domain/               # 业务逻辑层
│   ├── lib/                  # 工具库
│   ├── types/                # TypeScript 类型定义
│   └── i18n/                 # 国际化
│
├── src-tauri/                # Rust 后端
│   ├── src/                  # Rust 源码
│   │   ├── main.rs           # 主入口
│   │   ├── bundled_runtime.rs
│   │   ├── desktop_shell.rs
│   │   ├── gateway_supervisor.rs
│   │   ├── installer.rs
│   │   ├── ssh.rs
│   │   └── ssh_profiles.rs
│   ├── capabilities/         # Tauri v2 权限配置
│   ├── resources/            # 捆绑资源
│   │   └── vendor/           # Node.js + OpenClaw
│   ├── Cargo.toml            # Rust 依赖配置
│   ├── tauri.conf.json       # Tauri 主配置
│   └── tauri.windows.conf.json
│
├── tests/                    # 测试文件
├── docs/                     # 文档
├── scripts/                  # 构建脚本
│   ├── bundle-openclaw.mjs
│   └── bundle-node-runtime.mjs
│
├── package.json              # 前端依赖配置
├── pnpm-lock.yaml            # pnpm 锁文件
├── tsconfig.json             # TypeScript 配置
├── vite.config.ts            # Vite 配置
├── tailwind.config.js        # Tailwind 配置
└── .gitignore
```

---

## 3. 开发命令

### 3.1 前端开发

```bash
# 开发模式 (热重载)
pnpm dev

# 类型检查 + 构建
pnpm build:check

# 仅构建
pnpm build

# 预览构建结果
pnpm preview
```

### 3.2 Tauri 开发

```bash
# Tauri 开发模式 (前端 + Rust)
pnpm tauri dev

# Tauri 构建 (生产)
pnpm tauri build

# Windows 特定构建
pnpm tauri:build:windows

# macOS 特定构建
pnpm tauri:build:mac
```

### 3.3 资源捆绑

```bash
# 捆绑 OpenClaw
pnpm bundle:openclaw

# 捆绑 Node.js 运行时
pnpm bundle:node
```

### 3.4 测试

```bash
# 运行测试
pnpm test

# 测试.watch 模式
pnpm test -- --watch
```

---

## 4. 开发流程

### 4.1 新功能开发

```
1. 创建功能分支
   git checkout -b feature/your-feature-name

2. 开发功能
   - 前端：src/pages/, src/components/
   - 后端：src-tauri/src/
   - 类型：src/types/

3. 编写测试
   - 单元测试：*.test.ts
   - 集成测试：tests/

4. 提交代码
   git add .
   git commit -m "feat: description"

5. 推送到远程
   git push origin feature/your-feature-name
```

### 4.2 Bug 修复

```
1. 创建修复分支
   git checkout -b fix/issue-xxx

2. 修复 Bug
   - 定位问题
   - 编写修复代码
   - 添加回归测试

3. 提交代码
   git commit -m "fix: description of the bug fix"

4. 创建 PR
```

### 4.3 代码规范

#### Git Commit 规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

| 前缀 | 用途 | 示例 |
|------|------|------|
| `feat:` | 新功能 | `feat: add SSH profile management` |
| `fix:` | Bug 修复 | `fix: resolve gateway startup issue` |
| `docs:` | 文档更新 | `docs: update README.md` |
| `style:` | 代码格式 | `style: format with prettier` |
| `refactor:` | 重构 | `refactor: extract utility function` |
| `test:` | 测试 | `test: add unit tests for installer` |
| `chore:` | 杂项 | `chore: update dependencies` |

#### TypeScript 规范

```typescript
// ✅ 推荐：使用类型推断
const count = 0;  // 推断为 number

// ✅ 推荐：明确函数返回类型
function getUser(id: string): User | null {
  // ...
}

// ❌ 不推荐：使用 any
function process(data: any) { ... }

// ✅ 推荐：使用 unknown 代替 any
function process(data: unknown) {
  if (typeof data === 'string') {
    // 类型收窄
  }
}
```

#### Rust 规范

```rust
// ✅ 推荐：使用 Result 处理错误
fn load_config(path: &Path) -> Result<Config, String> {
    // ...
}

// ✅ 推荐：使用 Option 处理可选值
fn find_user(id: u32) -> Option<User> {
    // ...
}

// ❌ 不推荐：滥用 unwrap()
let config = load_config(path).unwrap();

// ✅ 推荐：使用 ? 操作符
let config = load_config(path)?;
```

---

## 5. 前端开发

### 5.1 组件开发

**基础组件** (基于 shadcn/ui)：
```tsx
// src/components/ui/button.tsx
import { Button } from "@/components/ui/button";

<Button variant="default" size="lg">
  点击我
</Button>
```

**页面组件**：
```tsx
// src/pages/OverviewPage.tsx
import { Card } from "@/components/ui/card";

export function OverviewPage() {
  return (
    <div className="page-grid">
      <Card>
        <CardHeader>
          <CardTitle>标题</CardTitle>
        </CardHeader>
        <CardContent>内容</CardContent>
      </Card>
    </div>
  );
}
```

### 5.2 状态管理 (Zustand)

```typescript
// src/stores/gatewayStore.ts
import { create } from 'zustand';

interface GatewayState {
  status: GatewayStatus;
  refresh: () => Promise<void>;
  restart: () => Promise<void>;
}

export const useGatewayStore = create<GatewayState>((set) => ({
  status: { state: 'stopped' },
  refresh: async () => { /* ... */ },
  restart: async () => { /* ... */ },
}));
```

**使用**：
```tsx
const { status, restart } = useGatewayStore();
```

### 5.3 路由 (React Router v7)

```tsx
// src/App.tsx
import { Routes, Route } from "react-router-dom";

<Routes>
  <Route index element={<OverviewPage />} />
  <Route path="/models" element={<ModelsPage />} />
  <Route path="/channels" element={<ChannelsPage />} />
  <Route path="/diagnostics" element={<DiagnosticsPage />} />
  <Route path="/settings" element={<SettingsPage />} />
</Routes>
```

### 5.4 国际化 (i18next)

```typescript
// src/i18n/locales/zh-CN/common.json
{
  "loading": "加载中...",
  "save": "保存",
  "cancel": "取消"
}
```

**使用**：
```tsx
import { useTranslation } from "react-i18next";

const { t } = useTranslation();
<button>{t('common:save')}</button>
```

---

## 6. Rust 后端开发

### 6.1 Tauri 命令

```rust
// src-tauri/src/main.rs
#[tauri::command]
fn greet(name: String) -> Result<String, String> {
    Ok(format!("Hello, {}!", name))
}

// 注册命令
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .build(tauri::generate_context!())
        .expect("error building tauri app")
        .run(|_app, _event| {});
}
```

### 6.2 前端调用

```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke("greet", { name: "World" });
```

### 6.3 事件监听

**Rust 发送事件**：
```rust
app.emit("status-changed", Status::Running)?;
```

**前端监听事件**：
```typescript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen("status-changed", (event) => {
  console.log("Status:", event.payload);
});
```

---

## 7. 调试

### 7.1 前端调试

**Chrome DevTools**：
```bash
pnpm dev
# 打开 http://localhost:1420
```

**React DevTools**：
- 安装 React Developer Tools 扩展
- 检查组件树和 props

### 7.2 Rust 调试

**日志输出**：
```rust
println!("Debug info: {}", value);
// 或
log::info!("Info: {}", value);
```

**查看日志**：
```bash
# macOS
log show --predicate 'process == "OpenClawSwitch"' --last 1m

# Windows
# 查看 Event Viewer

# Linux
journalctl -f
```

### 7.3 网络调试

```typescript
// 添加网络请求日志
console.log("API Request:", url, params);
```

---

## 8. 构建与打包

### 8.1 开发构建

```bash
# 开发模式
pnpm tauri dev
```

### 8.2 生产构建

```bash
# 全平台构建
pnpm tauri build

# Windows 构建
pnpm tauri:build:windows

# macOS 构建
pnpm tauri:build:mac
```

### 8.3 构建产物

```
src-tauri/target/release/
├── bundle/
│   ├── windows/
│   │   └── msi/
│   │       └── OpenClawSwitch_2.0.0_x64.msi
│   └── macos/
│       └── OpenClawSwitch.app
└── openclawswitch.exe
```

### 8.4 代码签名

**Windows**：
```toml
# src-tauri/tauri.windows.conf.json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_THUMBPRINT",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

**macOS**：
```bash
# 签名
codesign --sign "Developer ID Application: YOUR_NAME" \
  --options runtime \
  dist/OpenClawSwitch.app
```

---

## 9. 测试

### 9.1 单元测试

```typescript
// src/domain/installSteps.test.ts
import { describe, it, expect } from 'vitest';
import { getNextStep } from './installSteps';

describe('getNextStep', () => {
  it('should return next step', () => {
    expect(getNextStep('check-node')).toBe('install-fnm');
  });
});
```

### 9.2 运行测试

```bash
# 运行所有测试
pnpm test

# 运行特定文件
pnpm test -- installSteps.test.ts

# 覆盖率
pnpm test -- --coverage
```

---

## 10. 性能优化

### 10.1 前端优化

**代码分割**：
```tsx
const SettingsPage = lazy(() => import('./pages/SettingsPage'));
```

**记忆化**：
```tsx
const MemoizedComponent = React.memo(Component);
```

### 10.2 Rust 优化

**使用 release 模式**：
```bash
cargo build --release
```

**性能分析**：
```bash
cargo install cargo-flamegraph
cargo flamegraph --bin openclawswitch
```

---

## 11. 常见问题

### 11.1 开发环境问题

**Q: Rust 编译失败**
```bash
# 更新 Rust 工具链
rustup update

# 检查目标平台
rustup target list
```

**Q: Node 依赖安装失败**
```bash
# 清除缓存
pnpm store prune

# 重新安装
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

### 11.2 Tauri 问题

**Q: 开发模式空白页**
- 检查 Vite 端口 (默认 1420)
- 检查 `tauri.conf.json` 中的 `devUrl`

**Q: 构建产物过大**
- 启用代码分割
- 优化资源压缩
- 使用 `strip` 命令移除调试符号

---

*最后更新：2026-03-19*
