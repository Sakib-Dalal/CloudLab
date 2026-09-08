$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
npm ci
if ($LASTEXITCODE -ne 0) { throw 'npm ci failed' }
npm run build
if ($LASTEXITCODE -ne 0) { throw 'Frontend build failed' }
cargo build --release --locked -p cloudlab
if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
$CloudLabInstall = Join-Path $env:LOCALAPPDATA 'CloudLab'
New-Item -ItemType Directory -Force -Path "$CloudLabInstall\bin", "$CloudLabInstall\web" | Out-Null
Copy-Item 'target\release\cloudlab.exe' "$CloudLabInstall\bin\cloudlab.exe" -Force
Copy-Item 'dist\*' "$CloudLabInstall\web" -Recurse -Force
Write-Host "Installed. Run: & '$CloudLabInstall\bin\cloudlab.exe' serve --web-dir '$CloudLabInstall\web'"
Write-Host 'Use Docker Desktop with Linux containers for compute nodes.'
