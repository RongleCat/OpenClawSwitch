# 图标生成指南

由于项目模板需要图标文件,您有两个选择:

## 方式一: 使用占位图标 (快速)

运行此命令自动生成占位图标:

```bash
# 安装 Tauri 图标生成工具
cargo install tauri-cli

# 生成默认图标
cd src-tauri
cargo tauri icon path/to/your-icon.png
```

## 方式二: 手动准备图标

准备一个 1024x1024 的 PNG 图片,然后:

```bash
npm install -g @tauri-apps/cli
tauri icon path/to/your-icon.png
```

这会自动生成所有需要的图标格式:
- `icon.ico` (Windows)
- `icon.icns` (macOS)
- `32x32.png`, `128x128.png`, `128x128@2x.png` (通用)

## 方式三: 跳过图标 (临时方案)

如果暂时不需要自定义图标,可以使用 Tauri 默认图标:

1. 删除 `tauri.conf.json` 中的 `icon` 字段
2. 或保留现有占位符

## 推荐图标设计

- 尺寸: 1024x1024 px
- 格式: PNG (透明背景)
- 内容: 简洁的符号或 Logo
- 颜色: 高对比度

## 在线图标生成工具

- https://icon.kitchen/
- https://www.iconfinder.com/
- https://www.flaticon.com/

生成后,将图标保存为 `icon.png`,然后运行:

```bash
tauri icon icon.png
```
