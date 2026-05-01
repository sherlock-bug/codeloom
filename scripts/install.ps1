# CodeLoom 一键安装脚本 (Windows PowerShell)
$ErrorActionPreference = "Stop"

$BaseUrl = "https://github.com/xxx/codeloom/releases/latest/download"
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }

$BinaryName = "codeloom-windows-$Arch.exe"
$InstallDir = "$env:USERPROFILE\.codeloom\bin"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "Downloading codeloom for Windows/$Arch..."
$BinaryPath = "$InstallDir\codeloom.exe"
Invoke-WebRequest -Uri "$BaseUrl/$BinaryName" -OutFile "$BinaryPath"

# 添加到 PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
}

Write-Host ""
Write-Host "CodeLoom installed to $BinaryPath"
Write-Host ""
Write-Host "Quick start (open a new terminal):"
Write-Host "  codeloom index"
Write-Host "  codeloom status"
Write-Host "  opencode mcp add codeloom"
