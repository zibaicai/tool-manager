# 工具管理系统 - 一键部署脚本（面向新电脑 / 新用户）
# 完整流程：环境检测与安装 -> npm 依赖 -> 打包 -> 安装到用户目录 -> 桌面快捷方式 -> 配置初始化 -> 启动
# 入口：双击「一键部署.cmd」

param([switch]$Elevated)

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$BuildExe    = Join-Path $ProjectRoot 'src-tauri\target\release\tool-manager.exe'
$InstallDir  = Join-Path $env:LOCALAPPDATA 'Programs\ToolManager'
$InstallExe  = Join-Path $InstallDir 'tool-manager.exe'
$AppDataDir  = Join-Path $env:APPDATA 'com.toolmanager.app'
$AppName     = 'tool-manager'

function Write-Step($n, $msg) { Write-Host "`n[$n/8] $msg" -ForegroundColor Cyan }
function Die($msg) { Write-Host "`n[失败] $msg" -ForegroundColor Red; Read-Host "`n按回车键退出"; exit 1 }
function Ok($msg)  { Write-Host "  [OK] $msg" -ForegroundColor Green }

# 从注册表/环境变量重新加载 PATH（安装程序写入的 PATH 在当前窗口不会自动生效）
function Refresh-Path {
    $env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')
    $cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
    if ((Test-Path $cargoBin) -and ($env:Path -notlike "*$cargoBin*")) { $env:Path = "$cargoBin;$env:Path" }
}

function Test-Admin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    (New-Object Security.Principal.WindowsPrincipal($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-Command($name) { [bool](Get-Command $name -ErrorAction SilentlyContinue) }

# MSVC C++ 生成工具检测（cargo 在 Windows 上链接必需）
function Test-Msvc {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) { return $false }
    $inst = & $vswhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -latest -property installationPath 2>$null
    return [bool]$inst
}

# WebView2 运行时检测（Win11 自带，部分精简版 Win10 可能缺失）
function Test-WebView2 {
    $paths = @(
        'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
        'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    )
    foreach ($p in $paths) {
        $v = Get-ItemProperty $p -ErrorAction SilentlyContinue
        if ($v -and $v.pv) { return $true }
    }
    return $false
}

function Invoke-WingetInstall($id, $name) {
    Write-Host "  正在通过 winget 安装 $name ..." -ForegroundColor Yellow
    winget install --id $id -e --source winget --accept-source-agreements --accept-package-agreements --disable-interactivity
    if ($LASTEXITCODE -ne 0) { Die "安装 $name 失败（退出码 $LASTEXITCODE）。请手动安装后重新运行本脚本。" }
    Refresh-Path
}

# ---------- 0. 环境检测，需要安装系统级组件时自动提权 ----------
Refresh-Path
$needNode   = -not (Test-Command 'node')
$needMsvc   = -not (Test-Msvc)
$needWeb    = -not (Test-WebView2)
$needAdmin  = $needNode -or $needMsvc -or $needWeb

if ($needAdmin -and -not $Elevated -and -not (Test-Admin)) {
    Write-Host '检测到需要安装系统组件（Node.js / C++ 生成工具 / WebView2），将请求管理员权限...' -ForegroundColor Yellow
    try {
        Start-Process powershell -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Elevated"
    } catch {
        Die '已取消管理员授权，无法安装必需组件。'
    }
    exit 0
}

if (-not (Test-Command 'winget')) {
    Die '未找到 winget（应用安装程序）。请从微软商店安装「应用安装程序」后重试，或手动安装 Node.js 与 Visual Studio Build Tools。'
}

# ---------- 1. 安装 Node.js ----------
Write-Step 1 '检查 Node.js 环境'
if ($needNode) {
    Invoke-WingetInstall 'OpenJS.NodeJS.LTS' 'Node.js LTS'
    if (-not (Test-Command 'node')) { Die 'Node.js 安装后仍未找到，请重启电脑后重新运行本脚本。' }
}
Ok "Node.js $(node -v)"

# ---------- 2. 安装 Rust 工具链（rustup 按用户安装，无需管理员） ----------
Write-Step 2 '检查 Rust 工具链'
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (-not (Test-Path $cargoBin) -and -not (Test-Command 'cargo')) {
    Write-Host '  正在下载并安装 rustup（按用户安装，无需管理员）...' -ForegroundColor Yellow
    $init = Join-Path $env:TEMP 'rustup-init.exe'
    try {
        Invoke-WebRequest 'https://win.rustup.rs/x86_64' -OutFile $init -UseBasicParsing
    } catch {
        Die '下载 rustup-init 失败，请检查网络后重试，或手动安装 Rust：https://rustup.rs'
    }
    & $init -y --profile minimal --default-toolchain stable
    if ($LASTEXITCODE -ne 0) { Die 'Rust 安装失败，请重新运行本脚本。' }
    Refresh-Path
    Remove-Item $init -Force -ErrorAction SilentlyContinue
}
if (-not (Test-Command 'cargo')) { Die '未找到 cargo，请确认 Rust 安装成功后重启脚本。' }
Ok "cargo $(cargo --version)"

# ---------- 3. 安装 MSVC C++ 生成工具 ----------
Write-Step 3 '检查 C++ 生成工具（MSVC）'
if ($needMsvc) {
    Write-Host '  即将安装 Visual Studio 2022 Build Tools（含 MSVC 与 Windows SDK，体积较大，约 3~6GB，请耐心等待）...' -ForegroundColor Yellow
    winget install --id Microsoft.VisualStudio.2022.BuildTools -e --source winget `
        --accept-source-agreements --accept-package-agreements --disable-interactivity `
        --override '--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'
    if ($LASTEXITCODE -ne 0) {
        Die 'Visual Studio Build Tools 安装失败。请手动安装 Visual Studio Build Tools 并勾选「使用 C++ 的桌面开发」工作负载后重试。'
    }
    Refresh-Path
}
if (-not (Test-Msvc)) { Die '未检测到 MSVC 生成工具，请安装 Visual Studio Build Tools（C++ 桌面开发工作负载）后重试。' }
Ok 'MSVC C++ 生成工具就绪'

# ---------- 4. 安装 WebView2 运行时 ----------
Write-Step 4 '检查 WebView2 运行时'
if ($needWeb) {
    Invoke-WingetInstall 'Microsoft.EdgeWebView2Runtime' 'WebView2 Runtime'
}
if (-not (Test-WebView2)) { Die 'WebView2 运行时安装失败，请手动安装 Evergreen Runtime：https://developer.microsoft.com/microsoft-edge/webview2/' }
Ok 'WebView2 运行时就绪'

# ---------- 5. 安装前端依赖 ----------
Write-Step 5 '安装前端依赖（npm install）'
Push-Location $ProjectRoot
try {
    if (Test-Path (Join-Path $ProjectRoot 'node_modules')) {
        Write-Host '  node_modules 已存在，跳过（如需强制更新请先删除该目录）'
    } else {
        npm install
        if ($LASTEXITCODE -ne 0) { Die 'npm install 失败，请检查网络后重试。' }
    }

    # ---------- 6. 打包 ----------
    Write-Step 6 '打包应用（npx tauri build，首次较慢）'
    $running = Get-Process $AppName -ErrorAction SilentlyContinue
    if ($running) { $running | Stop-Process -Force; Start-Sleep -Seconds 2 }
    npx tauri build --no-bundle
    if ($LASTEXITCODE -ne 0) { Die '打包失败，请查看上方日志。' }
} finally {
    Pop-Location
}
if (-not (Test-Path $BuildExe)) { Die "未找到构建产物：$BuildExe" }
Ok '打包完成'

# ---------- 7. 安装到用户目录 + 桌面快捷方式 ----------
Write-Step 7 '安装程序并创建桌面快捷方式'
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item $BuildExe $InstallExe -Force
Ok "程序已安装：$InstallExe"

$desktop = [Environment]::GetFolderPath('Desktop')
$lnkPath = Join-Path $desktop '工具管理系统.lnk'
try {
    $ws = New-Object -ComObject WScript.Shell
    $sc = $ws.CreateShortcut($lnkPath)
    $sc.TargetPath = $InstallExe
    $sc.WorkingDirectory = $InstallDir
    $sc.IconLocation = "$InstallExe,0"
    $sc.Description = 'Tool Manager - 本地工具管理系统'
    $sc.Save()
    if (-not (Test-Path $lnkPath)) { throw '快捷方式文件未生成' }
    Ok "桌面快捷方式：$lnkPath"
} catch {
    # 退化方案：生成 .url 快捷方式（纯文本，不依赖 COM 权限）
    $urlPath = Join-Path $desktop '工具管理系统.url'
    "[InternetShortcut]`r`nURL=file:///$($InstallExe -replace '\\','/')" | Set-Content -Path $urlPath -Encoding ASCII
    Write-Host "  [警告] .lnk 创建失败，已改用 .url 快捷方式：$urlPath" -ForegroundColor Yellow
}

# ---------- 8. 初始化用户配置并启动 ----------
Write-Step 8 '初始化配置并启动'
New-Item -ItemType Directory -Force -Path $AppDataDir | Out-Null
$menuDst = Join-Path $AppDataDir 'menu.json'
if (-not (Test-Path $menuDst)) {
    # 新用户：写入不含任何个人路径的干净模板（与程序内置默认一致），scanRoot 首次启动后在应用内配置
    '{ "scanRoot": null, "categories": [] }' | Set-Content -Path $menuDst -Encoding UTF8
    Ok '已写入初始 menu.json'
}
$exeToolsDst = Join-Path $AppDataDir 'exe-tools.json'
if (-not (Test-Path $exeToolsDst)) {
    '{ "tools": [] }' | Set-Content -Path $exeToolsDst -Encoding UTF8
}

Start-Process -FilePath $InstallExe
Start-Sleep -Seconds 4
$alive = [bool](Get-Process $AppName -ErrorAction SilentlyContinue)

Write-Host ''
Write-Host '==================== 部署完成 ====================' -ForegroundColor Green
if ($alive) {
    Write-Host '程序已启动，桌面「工具管理系统」图标可随时打开。' -ForegroundColor Green
} else {
    Write-Host '程序文件已部署，但启动后未保持运行，请从桌面图标手动启动查看原因。' -ForegroundColor Yellow
}
Write-Host ''
Write-Host '重要提示：'
Write-Host "  配置目录：$AppDataDir"
try {
    $menu = Get-Content $menuDst -Raw | ConvertFrom-Json
    if (-not $menu.scanRoot) {
        Write-Host '  [提示] 尚未配置 CMD 工具扫描根目录：打开应用后点右上角「设置」按钮，选择你的 CMD 工具汇总目录即可。' -ForegroundColor Yellow
    } elseif (-not (Test-Path $menu.scanRoot)) {
        Write-Host "  [注意] menu.json 中的 scanRoot 当前为：$($menu.scanRoot)" -ForegroundColor Yellow
        Write-Host '         该路径在本机不存在，请点应用右上角「设置」改为你的 CMD 工具汇总目录。' -ForegroundColor Yellow
    }
} catch {}
Write-Host '  EXE 工具在应用内按分类手动录入即可；便携部署可在 tool-manager.exe 同目录建 config 文件夹。'
Write-Host '=================================================='

Read-Host "`n按回车键退出"
