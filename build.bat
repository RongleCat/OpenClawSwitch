@echo off
echo ========================================
echo OpenClawSwitch - Windows 构建脚本
echo ========================================
echo.

REM 检查 Node.js
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未检测到 Node.js,请先安装 Node.js 18+
    echo 下载地址: https://nodejs.org/
    pause
    exit /b 1
)

REM 检查 Rust
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未检测到 Rust,请先安装 Rust
    echo 下载地址: https://rustup.rs/
    pause
    exit /b 1
)

echo [1/4] 检查环境...
node --version
cargo --version

echo.
echo [2/4] 安装 npm 依赖...
call npm install
if %errorlevel% neq 0 (
    echo [错误] npm 依赖安装失败
    pause
    exit /b 1
)

echo.
echo [3/4] 构建应用...
echo 提示: 首次构建可能需要较长时间 (10-20分钟)
call npm run tauri build
if %errorlevel% neq 0 (
    echo [错误] 构建失败
    pause
    exit /b 1
)

echo.
echo [4/4] 构建完成!
echo.
echo ========================================
echo 可执行文件位置:
echo src-tauri\target\release\openclaw-manager.exe
echo.
echo 安装包位置:
echo src-tauri\target\release\bundle\
echo ========================================
echo.

pause
