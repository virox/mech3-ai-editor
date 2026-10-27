$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    npm run panel:tauri:build -- --no-bundle
    if ($LASTEXITCODE -ne 0) { throw "Industrial Panel build failed with exit code $LASTEXITCODE." }
    $releaseRoot = Join-Path (Split-Path -Parent $PSScriptRoot) 'release\industrial-panel'
    New-Item -ItemType Directory -Force -Path $releaseRoot | Out-Null
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'src-tauri\target\release\mech3-valve-switchboard.exe') -Destination (Join-Path $releaseRoot 'MechWarrior 3 Industrial Panel.exe') -Force
    Get-Item -LiteralPath (Join-Path $releaseRoot 'MechWarrior 3 Industrial Panel.exe')
} finally {
    Pop-Location
}
