# build-standalone.ps1 — Build aman binary standalone Nexora Finder
# Pemakaian: powershell -ExecutionPolicy Bypass -File .\build-standalone.ps1
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host ">> [1/4] Build frontend (dist/)" -ForegroundColor Cyan
npm.cmd run build
if ($LASTEXITCODE -ne 0) { throw "Frontend build gagal" }

Write-Host ">> [2/4] Build Rust release + custom-protocol (aset tertanam)" -ForegroundColor Cyan
# WAJIB: tanpa tauri/custom-protocol, exe akan memuat http://localhost:1420 -> jendela kosong abu-abu
cargo build --release --features tauri/custom-protocol --manifest-path "src-tauri\Cargo.toml"
if ($LASTEXITCODE -ne 0) { throw "Cargo build gagal" }

Write-Host ">> [3/4] Hentikan proses lama" -ForegroundColor Cyan
Get-Process nexora-finder -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

Write-Host ">> [4/4] Salin binary ke root proyek" -ForegroundColor Cyan
Copy-Item "D:\Rust\.nexora-target\release\nexora-finder.exe" ".\nexora-finder.exe" -Force
Get-Item ".\nexora-finder.exe" | Select-Object Name, Length, LastWriteTime
Write-Host ">> Selesai." -ForegroundColor Green
