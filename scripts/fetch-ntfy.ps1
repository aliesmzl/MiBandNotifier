# 下载 ntfy.exe（Windows 原生单二进制）到 tools 目录
# 用法: powershell -ExecutionPolicy Bypass -File scripts\fetch-ntfy.ps1
# 下载地址与校验方式参考官方文档 https://docs.ntfy.sh/install/

$ErrorActionPreference = "Stop"

$repo = "binwiederhier/ntfy"
$toolsDir = Join-Path $PSScriptRoot "..\tools"
$target = Join-Path $toolsDir "ntfy.exe"

if (Test-Path $target) {
    Write-Host "ntfy.exe 已存在: $target"
    & $target --version
    exit 0
}

Write-Host "获取最新版本号..."
$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" `
    -Headers @{ "User-Agent" = "MiBandNotifier" }
$version = $release.tag_name
Write-Host "最新版本: $version"

$assetName = "ntfy_${version}_windows_x86_64.zip"
$downloadUrl = "https://github.com/$repo/releases/download/$version/$assetName"
Write-Host "下载: $downloadUrl"

New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null
$zipPath = Join-Path $toolsDir $assetName
Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UserAgent "MiBandNotifier"

Write-Host "解压..."
Expand-Archive -Path $zipPath -DestinationPath $toolsDir -Force
# 压缩包结构: ntfy_<version>_windows_x86_64\ntfy.exe
$extracted = Join-Path $toolsDir "ntfy_${version}_windows_x86_64\ntfy.exe"
if (Test-Path $extracted) {
    Move-Item $extracted $target -Force
    Remove-Item (Join-Path $toolsDir "ntfy_${version}_windows_x86_64") -Recurse -Force
} else {
    # 兼容压缩包直接含 ntfy.exe 的情况
    $direct = Join-Path $toolsDir "ntfy.exe"
    if (-not (Test-Path $direct)) {
        throw "解压后未找到 ntfy.exe"
    }
}
Remove-Item $zipPath -Force

Write-Host "安装完成: $target"
& $target --version
