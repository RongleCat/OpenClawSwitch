# OpenClawSwitch 功能文档

> OpenClaw 桌面管理器 - 基于 Tauri v2 + React 19 构建

## 📚 文档导航

### 核心文档

| 文档 | 描述 |
|------|------|
| [架构设计](./01-architecture.md) | 技术架构、分层设计、数据流 |
| [核心功能](./02-core-features.md) | 工作台、模型配置、渠道管理等核心功能 |
| [UI 组件系统](./03-ui-components.md) | shadcn/ui 集成、组件列表、设计系统 |
| [SSH 远程管理](./04-ssh-management.md) | SSH 连接、配置管理、远程操作 |
| [安装与部署](./05-installation-deploy.md) | 环境检测、依赖安装、服务注册 |
| [插件扩展系统](./06-plugin-system.md) | 渠道插件、能力矩阵、扩展机制 |
| [开发指南](./07-development-guide.md) | 开发环境、命令、规范 |
| [测试规范](./08-testing.md) | 测试策略、测试用例、覆盖率 |

## 🚀 快速开始

### 项目简介

OpenClawSwitch 是 OpenClaw 的官方桌面管理应用，提供：

- **本地模式**：使用捆绑的 Node.js 和 OpenClaw 运行时
- **SSH 模式**：通过 SSH 远程管理服务器上的 OpenClaw

### 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | React 19 + TypeScript |
| UI 组件库 | shadcn/ui + Radix Primitives |
| 样式 | Tailwind CSS |
| 状态管理 | Zustand |
| 路由 | React Router v7 |
| 国际化 | react-i18next |
| 桌面框架 | Tauri v2.x |
| Rust 后端 | Rust 2021 Edition |

### 版本信息

- **当前版本**：2.0.0
- **OpenClaw 版本**：2026.3.7
- **许可证**：MIT

## 📁 项目结构

```
openclaw-manager-tauri/
├── src/                      # 前端源码 (React)
│   ├── pages/                # 页面组件
│   ├── components/           # 通用组件
│   ├── stores/               # 状态管理
│   ├── domain/               # 业务逻辑
│   └── lib/                  # 工具函数
├── src-tauri/                # Rust 后端源码
│   ├── src/                  # Rust 模块
│   ├── capabilities/         # Tauri v2 权限配置
│   └── resources/            # 捆绑资源
├── tests/                    # 测试文件
└── docs/                     # 文档
    └── features/             # 功能文档
```

## 🔗 相关链接

- [GitHub 仓库](https://github.com/RongleCat/OpenClawSwitch)
- [OpenClaw 项目](https://github.com/anthropics/openclaw)
- [Tauri 文档](https://tauri.app/)
- [React 文档](https://react.dev/)

---

*最后更新：2026-03-19*
