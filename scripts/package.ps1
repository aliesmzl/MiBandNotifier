# 打包发布：把 exe、脚本、文档、ntfy.exe 打成 zip
# 用法: powershell -ExecutionPolicy Bypass -File scripts\package.ps1
# 产物: dist\MiBandNotifier-<版本>-windows-x64.zip

param(
    # 仓库根目录；不传时按脚本位置/当前目录推断
    [string]$Root = ""
)

$ErrorActionPreference = "Stop"
if (-not $Root) {
    $scriptDir = $PSScriptRoot
    if (-not $scriptDir) {
        $invoked = $MyInvocation.MyCommand.Path
        if ($invoked) { $scriptDir = Split-Path $invoked -Parent }
    }
    if (-not $scriptDir) { $scriptDir = (Get-Location).Path }
    $Root = Split-Path $scriptDir -Parent
}
$version = "0.1.0"
$exe = Join-Path $Root "target\release\MiBandNotifier.exe"

if (-not (Test-Path $exe)) {
    Write-Host "未找到 $exe，先构建..."
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "构建失败" }
}

$stage = Join-Path $root "dist\MiBandNotifier"
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stage | Out-Null

# 主程序
Copy-Item $exe $stage

# 脚本与文档
Copy-Item (Join-Path $root "scripts") "$stage\scripts" -Recurse
Copy-Item (Join-Path $root "docs") "$stage\docs" -Recurse
Copy-Item (Join-Path $root "README.md") $stage

# ntfy.exe 一并打包（若存在），做到开箱即用
$ntfy = Join-Path $root "tools\ntfy.exe"
if (Test-Path $ntfy) {
    New-Item -ItemType Directory -Force -Path "$stage\tools" | Out-Null
    Copy-Item $ntfy "$stage\tools"
} else {
    Write-Host "提示: tools\ntfy.exe 不存在，打包后需用户自行运行 scripts\fetch-ntfy.ps1"
}

# 使用示例配置（不含任何密钥）
$exampleConfig = @"
toast = true

[events]
on_stop = true
on_permission = true
on_quota_warn = true

[ntfy]
server_url = "http://127.0.0.1:8090"
topic = ""

# 首次运行会自动生成随机 topic 并写回 %APPDATA%\MiBandNotifier\config.toml
[glm]
enabled = true
api_key = ""              # bigmodel.cn 控制台 -> API Keys
interval_minutes = 5
warn_threshold_percent = 80

[deepseek]
enabled = false
api_key = ""
interval_minutes = 30

[siliconflow]
enabled = false
api_key = ""
interval_minutes = 30
"@
New-Item -ItemType Directory -Force -Path "$stage\config-example" | Out-Null
Set-Content -Path "$stage\config-example\config.toml" -Value $exampleConfig -Encoding UTF8

$zipPath = Join-Path $root "dist\MiBandNotifier-$version-windows-x64.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path $stage -DestinationPath $zipPath
Remove-Item $stage -Recurse -Force

Write-Host "打包完成: $zipPath"
Get-Item $zipPath | Select-Object Name, Length
