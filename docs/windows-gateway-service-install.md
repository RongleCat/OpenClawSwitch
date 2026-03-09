# Windows 网关服务安装失败说明

日期：2026-03-08

## 问题现象

Windows 上执行“安装并启动网关”时，界面提示：

- `网关服务安装失败: 命令执行失败，退出码: Some(1)`

失败发生在内置 `nssm.exe` 注册 `OpenClaw Gateway` 服务这一步。

## 根因

根因是 Windows 服务安装命令仍然走了整串 shell 拼接：

1. `nssm.exe` 的路径可能位于带空格目录；
2. `gateway-service.cmd` 脚本路径也可能位于带空格目录；
3. 旧实现先把这些值 `shell_quote()`，再拼成一整条命令交给 `cmd /c`；
4. 同时 `cmd.exe /c` 的脚本路径没有作为“带引号的单个参数”传给 `cmd.exe`；
5. 最终 `nssm` 或 `cmd.exe` 会把路径拆坏，直接返回退出码 `1`。

## 修复原则

Windows 服务安装相关命令必须遵守下面两个约束：

- `nssm.exe` 必须通过 `Command::new()` + 参数数组执行，不能再拼整串 shell 文本；
- `cmd.exe /c` 执行脚本时，脚本路径必须作为一个单独参数传入，并保留内部引号。

## 当前保护

已添加回归测试：

- `src-tauri/src/installer.rs` 中的 `nssm_command_keeps_executable_and_script_paths_as_single_args`

这个测试保证：

- `nssm.exe` 自身路径不会因为空格被拆开；
- `OpenClaw Gateway` 服务名保持单参数；
- `gateway-service.cmd` 会以 `"<path>"` 形式作为 `cmd /c` 的单个参数传入。
