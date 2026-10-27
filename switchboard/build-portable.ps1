$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$releaseRoot = Join-Path $projectRoot 'release\switchboard'
npm run tauri:build -- --no-bundle
if ($LASTEXITCODE -ne 0) { throw "Stable Switchboard build failed with exit code $LASTEXITCODE." }
New-Item -ItemType Directory -Force -Path $releaseRoot | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'src-tauri\target\release\mech3-valve-switchboard.exe') -Destination (Join-Path $releaseRoot 'MechWarrior 3 Valve Switchboard.exe') -Force
Get-Item -LiteralPath (Join-Path $releaseRoot 'MechWarrior 3 Valve Switchboard.exe')
