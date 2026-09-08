# 把 tools\ntfy.exe 注册为 Windows 服务（ntfy 官方推荐的 Windows 部署方式）。
# 服务监听 0.0.0.0:8090：本机推送 + 局域网手机订阅都需要。
# 需要管理员权限运行：右键"使用管理员身份运行" PowerShell 后执行本脚本。
#
# 用法: powershell -ExecutionPolicy Bypass -File scripts\setup-ntfy-service.ps1
# 卸载: powershell -ExecutionPolicy Bypass -File scripts\setup-ntfy-service.ps1 -Uninstall

param(
    [switch]$Uninstall,
    [int]$Port = 8090
)

$ErrorActionPreference = "Stop"
$serviceName = "MiBandNotifierNtfy"
$exe = Join-Path $PSScriptRoot "..\tools\ntfy.exe"
$exe = [System.IO.Path]::GetFullPath($exe)

if ($Uninstall) {
    Write-Host "停止并删除服务 $serviceName ..."
    if (Get-Service $serviceName -ErrorAction SilentlyContinue) {
        Stop-Service $serviceName -Force
        sc.exe delete $serviceName
    } else {
        Write-Host "服务不存在，无需卸载"
    }
    exit 0
}

if (-not (Test-Path $exe)) {
    throw "未找到 $exe，请先运行 scripts\fetch-ntfy.ps1"
}

# 管理员权限检查
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    throw "需要管理员权限：请右键以管理员身份运行 PowerShell 再执行本脚本"
}

# 已存在则先删除（升级场景）
if (Get-Service $serviceName -ErrorAction SilentlyContinue) {
    Write-Host "服务已存在，先删除旧版本..."
    Stop-Service $serviceName -Force
    sc.exe delete $serviceName | Out-Null
    Start-Sleep -Seconds 1
}

Write-Host "创建服务 $serviceName（监听 0.0.0.0:$Port）..."
$binPath = "`"$exe`" serve --listen-http :$Port"
sc.exe create $serviceName binPath= $binPath start= auto DisplayName= "MiBandNotifier ntfy 服务" | Out-Null
sc.exe description $serviceName "MiBandNotifier 的局域网通知服务（ntfy），监听 0.0.0.0:$Port"

Write-Host "启动服务..."
sc.exe start $serviceName | Out-Null
Start-Sleep -Seconds 2
Get-Service $serviceName

Write-Host ""
Write-Host "完成。防火墙放行（如未放行）: "
Write-Host "  netsh advfirewall firewall add rule name=`"MiBandNotifier ntfy`" dir=in action=allow protocol=TCP localport=$Port"
Write-Host ""
Write-Host "健康检查: curl http://127.0.0.1:$Port/v1/health"
