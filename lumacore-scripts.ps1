# Root entrypoint for the script launcher TUI (Windows).
# On Unix, use ./lumacore-scripts instead.

param(
    [switch]$Release,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $RepoRoot

if ($Help) {
    Write-Host @"
LumaCore scripts launcher

Usage:
  .\lumacore-scripts.ps1
  .\lumacore-scripts.ps1 -Release

This launches the ratatui menu for the canonical scripts.
  Unix scripts:   scripts/unix/
  Windows scripts: scripts/win/
The launcher builds the matching workspace binary in target/ if needed.
"@
    Pop-Location
    exit 0
}

if ($Release) {
    $Profile = "release"
    $Bin = ".\target\release\lumacore-scripts.exe"
    $BuildArgs = @("--release")
} else {
    $Profile = "debug"
    $Bin = ".\target\debug\lumacore-scripts.exe"
    $BuildArgs = @()
}

if (-not (Test-Path $Bin)) {
    Write-Host "Building lumacore-scripts ($Profile)..."
    cargo build @BuildArgs -p lumacore-scripts 2>&1 |
        Select-String -Pattern "(Compiling lumacore|Finished|error)"
}

if (-not (Test-Path $Bin)) {
    Write-Error "Failed to build $Bin"
    Pop-Location
    exit 1
}

& $Bin @args
Pop-Location
