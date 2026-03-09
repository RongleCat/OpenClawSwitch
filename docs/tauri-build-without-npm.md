# Tauri 打包前命令不再依赖 npm

日期：2026-03-08

## 问题现象

执行 `tauri build` 时，`beforeBuildCommand` 会先跑前端和资源准备步骤。

此前这里写死的是：

- `npm run vendor:openclaw`
- `npm run vendor:windows-tools`
- `npm run build`

在当前 Windows 环境下，`npm` 被 Version Fox 的临时 Node 安装包装过，而对应的 `npm-cli.js` / `npm-prefix.js` 已经不存在，所以打包在真正进入 Rust/Tauri 之前就直接失败。

## 根因

根因不是项目脚本本身有问题，而是 `tauri.conf.json` 里的前置命令对 `npm run` 有硬依赖。

只要外部 `npm` 包装层损坏，即使：

- `node` 本身可用；
- `scripts/*.mjs` 可正常执行；
- 本地 `node_modules` 完整；

打包也会提前失败。

## 修复原则

`tauri` 的前置命令应直接调用本地可执行入口，而不是依赖外部包管理器包装层。

当前约束：

- `beforeDevCommand` 直接运行本地 `vite` CLI JS 入口；
- `beforeBuildCommand` 直接运行 `node ./scripts/*.mjs` 和本地 `vite` CLI JS 入口；
- 不再在 `tauri.conf.json` 中使用 `npm run ...`。

## 回归保护

已添加回归测试：

- `tests/tauriConfig.test.ts`

这个测试保证：

- `beforeDevCommand` 不包含 `npm run`；
- `beforeBuildCommand` 不包含 `npm run`。
