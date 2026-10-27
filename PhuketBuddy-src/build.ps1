# Run from Windows with Rust stable and Visual Studio C++ Build Tools installed.
$ErrorActionPreference = 'Stop'
$projectRoot = $PSScriptRoot
$outputDir = Join-Path (Split-Path $projectRoot -Parent) 'dist\windows'

Push-Location (Join-Path $projectRoot 'src-tauri')
try {
    cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Rust build failed.' }
    $exe = Join-Path $PWD 'target\release\phuketbuddy.exe'
    if (!(Test-Path $exe)) { throw 'Build output not found.' }
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
    $app = Join-Path $outputDir 'PhuketBuddy.exe'
    Copy-Item $exe $app -Force
    Copy-Item (Join-Path $projectRoot 'README.md') (Join-Path $outputDir 'README.md') -Force
    $archive = Join-Path (Split-Path $outputDir -Parent) 'PhuketBuddy-v0.1.0-Windows.zip'
    Compress-Archive -Path $app, (Join-Path $outputDir 'README.md') -DestinationPath $archive -Force
    Write-Host "Built: $app"
    Write-Host "Archive: $archive"
} finally {
    Pop-Location
}
