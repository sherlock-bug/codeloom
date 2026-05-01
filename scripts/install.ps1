# CodeLoom 一键安装脚本 (Windows PowerShell)
# 用法:
#   irm https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.ps1 | iex
#   irm ... | iex -args "--from-source"   # 从源码编译安装

param(
    [switch]$FromSource
)

$ErrorActionPreference = "Stop"
$InstallDir = "$env:USERPROFILE\.codeloom\bin"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

if ($FromSource) {
    Write-Host "=== 从源码编译安装 CodeLoom ==="
    $Repo = "https://github.com/sherlock-bug/codeloom.git"
    $TmpDir = Join-Path $env:TEMP "codeloom-src-$(Get-Random)"
    git clone --depth 1 $Repo $TmpDir
    Push-Location $TmpDir
    cargo build --release
    Copy-Item target\release\codeloom.exe "$InstallDir\codeloom.exe"
    Pop-Location
    Remove-Item -Recurse -Force $TmpDir
} else {
    $BaseUrl = "https://github.com/sherlock-bug/codeloom/releases/latest/download"
    $Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
    $BinaryName = "codeloom-windows-$Arch.exe"

    Write-Host "Downloading codeloom for Windows/$Arch..."
    $BinaryPath = "$InstallDir\codeloom.exe"
    Invoke-WebRequest -Uri "$BaseUrl/$BinaryName" -OutFile $BinaryPath
}

# 添加到 PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
}

Write-Host ""
Write-Host "CodeLoom installed to $InstallDir\codeloom.exe"
Write-Host ""
Write-Host "Quick start (open a new terminal):"
Write-Host "  codeloom check"
Write-Host "  codeloom index ."
Write-Host "  codeloom status"
