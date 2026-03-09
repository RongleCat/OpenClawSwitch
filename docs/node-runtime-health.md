# Node 运行环境健康度说明

日期：2026-03-08

## 问题现象

Windows 上可能出现这种情况：

- `node --version` 正常；
- `npm --version` 失败；
- 失败原因通常是外部 Node 管理器（例如 Version Fox）留下了损坏的 `npm-cli.js` / `npm-prefix.js` 包装层。

在这种环境里，如果安装器只检查 `node` 版本，就会误判“Node.js 已满足要求”，然后继续使用坏掉的 `npm.cmd`。

## 根因

此前安装器的 `check_node_installed()` 只判断：

- `node` 是否存在；
- 主版本是否 >= 22。

但这还不够。对于 OpenClaw 安装流程来说，真正需要的是：

- `node` 可执行；
- `npm` 也必须可执行。

只要 `npm` 坏了，后续安装 tarball、全局安装 OpenClaw、安装扩展都会失败。

## 当前修复

现在“Node.js 满足要求”的判定已经改成：

- `node` 主版本 >= 22；
- 且活跃运行时对应的 `npm` 真正可执行。

在 Windows 上，安装器会优先检查：

- 当前活跃 Node 目录里的 `npm.cmd`；
- 如果不可用，再检查 PATH 上的 `npm.cmd`。

如果 `node` 版本达标但 `npm` 不可用，安装器不会再跳过，而是会转去安装托管 Node.js 来修复环境。

## 回归保护

已添加回归测试：

- `src-tauri/src/installer.rs` 中的 `node_with_broken_npm_does_not_meet_requirement`

这个测试保证：

- `v24 + npm 可用` => 满足要求；
- `v24 + npm 不可用` => 不满足要求；
- `v20 + npm 可用` => 仍然不满足要求。
