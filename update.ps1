# 工具管理系统一键更新脚本
# 流程：重新构建（含前端打包）-> 结束运行中的旧版 -> 覆盖安装目录 -> 重新启动
# 用法：双击「更新工具管理系统.cmd」，或在项目根目录执行 powershell -ExecutionPolicy Bypass -File .\update.ps1

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$BuildExe    = Join-Path $ProjectRoot 'src-tauri\target\release\tool-manager.exe'
$InstallDir  = Join-Path $env:LOCALAPPDATA 'Programs\ToolManager'
$InstallExe  = Join-Path $InstallDir 'tool-manager.exe'
$AppName     = 'tool-manager'

function Write-Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Die($msg) { Write-Host "`n[失败] $msg" -ForegroundColor Red; Read-Host "`n按回车键退出"; exit 1 }

# 1. 补齐 PATH（旧终端可能没有 cargo / node）
$env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')

if (-not (Get-Command npx -ErrorAction SilentlyContinue)) { Die '未找到 npx（Node.js 未安装或不在 PATH）' }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Die '未找到 cargo（Rust 工具链未安装或不在 PATH）' }

# 2. 结束运行中的旧版（避免 exe 被占用导致覆盖失败）
Write-Step '结束运行中的旧版程序'
$running = Get-Process $AppName -ErrorAction SilentlyContinue
if ($running) {
    $running | Stop-Process -Force
    Start-Sleep -Seconds 2
    Write-Host '已结束旧版进程'
} else {
    Write-Host '当前没有运行中的程序'
}

# 3. 重新构建（tauri CLI 会先执行 npm run build，再以独立模式编译 exe）
Write-Step '开始构建（首次/依赖变更时较慢，请耐心等待）'
Push-Location $ProjectRoot
try {
    & npx tauri build --no-bundle
    if ($LASTEXITCODE -ne 0) { Die '构建失败，请查看上方日志' }
} finally {
    Pop-Location
}

if (-not (Test-Path $BuildExe)) { Die "未找到构建产物：$BuildExe" }

# 4. 覆盖安装目录中的 exe
Write-Step '更新安装目录'
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item $BuildExe $InstallExe -Force
Write-Host "已更新：$InstallExe"

# 同步菜单配置：项目 config\menu.json -> 用户配置目录
# （安装版只读 %APPDATA% 下的配置；exe-tools.json 由应用自己维护，绝不同步以免覆盖录入数据）
$menuSrc = Join-Path $ProjectRoot 'config\menu.json'
$menuDst = Join-Path $env:APPDATA 'com.toolmanager.app\menu.json'
if (Test-Path $menuSrc) {
    New-Item -ItemType Directory -Force -Path (Split-Path $menuDst) | Out-Null
    Copy-Item $menuSrc $menuDst -Force
    Write-Host '已同步 menu.json 到用户配置目录'
}

# 5. 重新启动
Write-Step '启动新版本'
Start-Process -FilePath $InstallExe
Start-Sleep -Seconds 3
if (Get-Process $AppName -ErrorAction SilentlyContinue) {
    Write-Host "`n[完成] 更新成功，新版本已启动。" -ForegroundColor Green
} else {
    Write-Host "`n[警告] exe 已更新，但启动后进程未保持，请手动从桌面图标启动排查。" -ForegroundColor Yellow
}

Read-Host "`n按回车键退出"
