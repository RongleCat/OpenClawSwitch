# Windows Git 安装流程说明

日期：2026-03-08

## 官方现状

参考的官方安装链路有两条：

- OpenClaw 官方 `install.ps1` / `install.cmd`：Git 模式下只检查 `git` 是否在 PATH 中；如果缺失，直接提示用户去手动安装 Git，然后退出。
- Git for Windows 官方发布：除了标准安装器，也提供适合嵌入式/便携场景的 `MinGit` 压缩包。

这意味着官方 OpenClaw 脚本本身并没有解决“新 Windows 机器没有 Git”这个问题，而是把问题交给用户手动处理。

## 我们的目标

桌面安装器要尽量做到一键安装，因此 Windows 上的 Git 流程改为：

1. 优先使用安装包内置的 `MinGit` 离线包。
2. 如果离线包缺失或损坏，尝试使用随包元数据里的国内镜像地址下载。
3. 国内镜像失败后，再回退到 Git for Windows 官方发布地址。
4. 上述都失败后，最后再尝试 `winget install Git.Git`。
5. 只有所有兜底都失败时，才阻断安装流程并报错。

## 资源布局

Windows 安装包内的 Git 离线资源约定放在：

- `src-tauri/resources/windows/git/mingit.zip`
- `src-tauri/resources/windows/git/metadata.json`

`metadata.json` 用于记录：

- `sourceUrl`：国内镜像下载地址
- `officialUrl`：Git for Windows 官方下载地址
- `assetName`：MinGit 原始文件名
- `release`：对应发布版本

## 运行时行为

安装器会把解压后的 Git 放到用户目录下的托管运行时中，并把对应目录前置到用户 PATH：

- `~/.openclaw/runtime/git/mingit`

这样做有两个好处：

- 不依赖系统是否已经装过 Git。
- 即使系统 PATH 里有损坏或不可用的旧 Git，也会优先命中我们托管的健康 Git。

## 回归关注点

后续改这块时，必须保证：

- 不能只依赖 `winget`
- 不能发现 Git 缺失就直接阻断安装
- 不能把托管 Git 只写入当前进程 PATH，而不持久化到用户 PATH
- 不能让系统里已有但损坏的 Git 抢到优先级
