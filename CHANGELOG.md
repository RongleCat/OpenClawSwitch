# Changelog

## [1.2.0] - 2026-02-26

### ✨ 新功能

- **粘贴配置导入**：添加服务商时支持直接粘贴 JSON 配置，自动解析提取服务商信息和模型列表
- **智能 JSON 解析**：支持多种非标准 JSON 格式，包括 `models.providers` 嵌套、`providers` 片段、`name: provider` 片段等
- **分段器切换**：添加服务商弹窗新增「手动配置 / 粘贴配置」Tab 切换，编辑模式始终为手动回填
- **API Key 双向绑定**：粘贴模式下 API Key 独立输入框与 JSON 配置双向同步
- **名称智能提取**：从 JSON 结构中自动提取服务商名称，无法提取时由用户手动输入

### 🐛 修复

- 修复删除服务商时确认弹窗弹出的同时服务商已被删除的问题（替换原生 `confirm` 为 Tauri `ask` 对话框）
- SSH 模式下禁用「重启网关」和「打开 TUI」按钮，避免无效操作

### 🔧 技术细节

- 新增 Rust 后端命令 `import_provider`，支持一次性导入包含 models 数组的完整服务商配置
- 新增 `src/utils/parseProviderJson.ts` 纯函数，从 App.vue 中提取解析逻辑便于测试
- 新增 `tests/parseProviderJson.test.ts` 单元测试（10 个用例），覆盖所有 JSON 格式变体
- 引入 vitest 测试框架

## [1.1.0] - 2026-02-25

### ✨ 新功能

- **SSH 远程连接**：支持通过 SSH 连接 Linux 服务器，远程管理 OpenClaw 配置文件
- **SSH 认证**：支持密码和私钥两种认证方式，含指纹验证确认
- **连接配置管理**：保存/加载/删除 SSH 连接配置，密码随配置一并保存
- **远程文件浏览器**：自动搜索远程配置文件 + 手动目录浏览，支持面包屑导航和路径跳转
- **远程配置读写**：通过 SSH channel 命令读取和写入远程配置文件
- **SSH 保存按钮**：SSH 模式下显示醒目的「保存到远程」按钮，手动控制保存时机

### 🐛 修复

- 修复断开 SSH 连接后左侧栏消失的问题（保留配置数据，仅重置连接状态）
- 修复远程文件浏览器前进/后退导航的异步竞态问题
- 修复远程 shell（zsh/oh-my-zsh）注入 ANSI/OSC 转义序列导致文件名乱码
- 使用 SSH channel 命令替代 SFTP 子系统，解决 SFTP 超时兼容性问题

### 🔧 技术细节

- 新增 Rust SSH 模块 `ssh.rs`，基于 `ssh2` crate 实现连接管理
- 所有远程文件操作通过 `channel.exec()` 执行（`ls -la`、`cat`、`test -f`）
- 新增 ANSI 转义序列清理函数 `strip_ansi_escapes`
- 新增前端组件：`SshConnectModal`、`SshFingerprintDialog`、`RemoteFileBrowser`
- 新增 SSH 相关类型定义：`SshProfile`、`FingerprintInfo`、`RemoteFileEntry`

## [1.0.0] - 2025-02-01

### 初始版本

- OpenClaw 配置文件可视化管理
- 服务商增删改查
- 模型管理与主/备模型选择
- 本地配置文件自动保存
- 网关重启、TUI 打开等工具集成
- Minimax 国服修复
- 源文件查看
