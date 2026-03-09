# Windows OpenClaw 安装路径转义说明

日期：2026-03-08

## 问题现象

Windows 安装内置 `openclaw.tgz` 时，日志里会出现类似下面的错误路径：

```text
...\target\release\bundle\msi\"C:\Program\package.json
```

`npm` 最终把 `\"C:\Program Files\...\openclaw.tgz` 识别成了错误路径，因此报 `ENOENT`，并继续尝试去找不存在的 `package.json`。

## 根因

根因不是 tarball 文件缺失，而是 **Windows 下路径被重复转义**：

1. 安装器先对 tarball 路径调用了 `shell_quote()`；
2. 然后又把整条命令拼成字符串，交给 `cmd /c` 执行；
3. Windows shell 会把 `\"` 当成字面字符的一部分；
4. 最终 `npm` 收到的不是一个正常文件路径，而是一个带错误前缀的参数。

## 修复原则

这类本地文件路径安装 **不要再走整串 shell 拼接**，而是：

- 使用 `Command::new()`；
- 用 `args([...])` 逐个传参；
- 仅在日志展示时做可读性引号包装；
- 安装逻辑接收原始路径，不接收预先 `shell_quote()` 的字符串。

## 当前保护

本次修复已经加入回归测试：

- `src-tauri/src/installer.rs` 中的 `openclaw_install_command_keeps_tarball_path_as_single_arg`

这个测试专门保证：

- tarball 路径作为单独参数传给 `npm.cmd`；
- 日志里可以看到带引号的可读命令；
- 实际执行不再把 `"` 写进参数值。

## 后续约束

如果以后再改 OpenClaw 安装流程，请保持下面这个约束不变：

> 带空格的 Windows 本地路径，只能作为 `Command` 参数传递，不能先 `shell_quote()` 再拼进 `cmd /c` 字符串。
