$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$releaseRoot = Join-Path $projectRoot 'release\switchboard-dev'
npm run tauri:build
if ($LASTEXITCODE -ne 0) { throw "Developer Switchboard build failed with exit code $LASTEXITCODE." }
New-Item -ItemType Directory -Force -Path $releaseRoot | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'src-tauri\target\release\mech3-valve-switchboard-dev.exe') -Destination (Join-Path $releaseRoot 'MechWarrior 3 Valve Switchboard Dev.exe') -Force
Get-Item -LiteralPath (Join-Path $releaseRoot 'MechWarrior 3 Valve Switchboard Dev.exe')
