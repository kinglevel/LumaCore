# Connect to an existing lumacore-server without spawning or restarting it (Windows).
#
# Usage:
#   .\scripts\connect.ps1
#   .\scripts\connect.ps1 -User LD
#   .\scripts\connect.ps1 -Target 192.168.1.20
#   .\scripts\connect.ps1 -Cmd "list fixture"

param(
    [switch]$DebugBuild,
    [switch]$Ship,
    [string]$User   = "Admin",
    [string]$Target = "127.0.0.1",
    [string]$Cmd    = "",
    [switch]$Help
)

# Ensure cargo is in PATH (Rustup default install location)
$CargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "$env:USERPROFILE\.cargo" }
$CargoBin  = Join-Path $CargoHome "bin"
if ((Test-Path $CargoBin) -and ($env:PATH -notlike "*$CargoBin*")) {
    $env:PATH = "$CargoBin;$env:PATH"
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot  = Split-Path -Parent (Split-Path -Parent $ScriptDir)

Push-Location $RepoRoot

if ($DebugBuild) {
    $Profile   = "dev"
    $CliBin    = ".\target\debug\lumacore-cli.exe"
    $BuildArgs = @()
} elseif ($Ship) {
    $Profile   = "release"
    $CliBin    = ".\target\release\lumacore-cli.exe"
    $BuildArgs = @("--release")
} else {
    $Profile   = "local"
    $CliBin    = ".\target\local\lumacore-cli.exe"
    $BuildArgs = @("--profile", "local")
}

if (-not (Test-Path $CliBin)) {
    Write-Host "Building LumaCore CLI ($Profile)..."
    cmd /c "cargo build $($BuildArgs -join ' ') -p lumacore-cli 2>&1"
    if (-not (Test-Path $CliBin)) {
        Write-Host "Build failed!" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

if ($Help) {
    & $CliBin --help
    Pop-Location
    exit 0
}

$cliArgs = @("--target", $Target, "--user", $User)
if ($Cmd -ne "") {
    $cliArgs += "--cmd", $Cmd
}

& $CliBin @cliArgs
Pop-Location
