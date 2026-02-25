# Changelog

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
