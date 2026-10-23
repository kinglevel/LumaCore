# The plan runner (Windows twin of scripts/unix/dev): build what you name,
# then run what you name, in a fixed order. The lumacore-scripts launcher's
# Plan pane writes this command line; start_server and start_developer are
# presets of it.
#
# Usage:
#   .\scripts\win\dev.ps1 [-Stop] [-Build server,cli,app] [-Tracy]
#                         [-Run server,tracy,meownitor,web,app,cli-window,cli]
#                         [-Show demo] [-User Admin] [-CleanStart] [-DebugBuild]
#                         [-Cmd "list fixture"]
#
# Steps: stop, build (server, cli, app), server (web UI from
# apps\lumacore-app\frontend, wait for health), Tracy, Meownitor, web UI,
# app, CLI window, then the CLI here last. The first failure stops the plan;
# a binary a run step needs but nobody built is built first.

param(
    [switch]$Stop,
    [string]$Build = "",
    [switch]$Tracy,
    [string]$Run = "",
    [string]$Show = "",
    [string]$User = "Admin",
    [switch]$CleanStart,
    [switch]$DebugBuild,
    [switch]$Ship,
    [string]$Cmd = "",
    [switch]$Help
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot  = Split-Path -Parent (Split-Path -Parent $ScriptDir)
if ($Help) { Get-Content $MyInvocation.MyCommand.Path | Select-Object -Skip 0 -First 15; exit 0 }
Push-Location $RepoRoot

$CargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "$env:USERPROFILE\.cargo" }
$CargoBin  = Join-Path $CargoHome "bin"
if ((Test-Path $CargoBin) -and ($env:PATH -notlike "*$CargoBin*")) {
    $env:PATH = "$CargoBin;$env:PATH"
}

# The `local` profile by default (decisions §34 "Build times hardened" 2); -Ship = full release.
if ($DebugBuild) {
    $Profile = "dev"; $TargetDir = ".\target\debug"; $BuildArgs = @()
} elseif ($Ship) {
    $Profile = "release"; $TargetDir = ".\target\release"; $BuildArgs = @("--release")
} else {
    $Profile = "local"; $TargetDir = ".\target\local"; $BuildArgs = @("--profile", "local")
}
$ServerBin = "$TargetDir\lumacore-server.exe"
$CliBin    = "$TargetDir\lumacore-cli.exe"
$AppBin    = "$TargetDir\lumacore-app.exe"
$ServerLog = "$TargetDir\server.log"
$WebRoot   = if ($env:LC_WEB_ROOT) { $env:LC_WEB_ROOT } else { Join-Path $RepoRoot "crates\ui\lumacore-webui\web" }
$HttpPort  = 47656

function Fail($msg) { Write-Host "x $msg" -ForegroundColor Red; Pop-Location; exit 1 }

$builds = @{}
foreach ($b in ($Build -split ',' | Where-Object { $_ -ne "" })) {
    switch ($b) {
        { $_ -in "server", "cli", "app" } { $builds[$b] = $true }
        "all" { $builds["server"] = $true; $builds["cli"] = $true; $builds["app"] = $true }
        default { Fail "Unknown -Build item: $b (server, cli, app)" }
    }
}
$runs = @{}
foreach ($r in ($Run -split ',' | Where-Object { $_ -ne "" })) {
    if ($r -in "server", "tracy", "meownitor", "web", "app", "cli-window", "cli") { $runs[$r] = $true }
    else { Fail "Unknown -Run item: $r (server, tracy, meownitor, web, app, cli-window, cli)" }
}
if ($runs["tracy"]) { $Tracy = $true }
if ($Cmd -ne "" -and -not $runs["cli"]) { Fail "-Cmd needs -Run cli" }
if ($builds.Count -eq 0 -and $runs.Count -eq 0 -and -not $Stop) { Fail "Nothing to do: name -Build and/or -Run items" }

function Needs($what, $bin) {
    if (-not $builds[$what] -and -not (Test-Path $bin)) {
        Write-Host "$bin isn't built - building it for -Run"
        $builds[$what] = $true
    }
}
if ($runs["server"]) { Needs "server" $ServerBin; Needs "cli" $CliBin }
foreach ($c in "meownitor", "cli-window", "cli") { if ($runs[$c]) { Needs "cli" $CliBin } }
if ($runs["app"]) { Needs "app" $AppBin }

function Server-Healthy {
    if (-not (Test-Path $CliBin)) { return $false }
    cmd /c "`"$CliBin`" --target 127.0.0.1 --health 2>nul" | Out-Null
    return ($LASTEXITCODE -eq 0)
}

# -- 1. Stop -----------------------------------------------------------------
if ($Stop) {
    & (Join-Path $ScriptDir "stop_developer.ps1")
    if ($LASTEXITCODE -ne 0) { Fail "Stop failed" }
}

# -- 2. Build ----------------------------------------------------------------
# A running server or app holds its .exe: stop what gets rebuilt first.
if ($builds["server"] -and $runs["server"]) {
    Get-Process -Name "lumacore-server" -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Milliseconds 500
}
function Cargo($what, $cargoArgs) {
    Write-Host "Building $what ($Profile)..."
    cmd /c "cargo build $($BuildArgs -join ' ') $cargoArgs 2>&1"
    if ($LASTEXITCODE -ne 0) { Fail "Build failed!" }
}
if ($builds["server"]) {
    $features = "lumacore/perfetto,lumacore/http"
    if ($Tracy) { $features += ",lumacore/tracy" }
    Cargo "lumacore-server" "-p lumacore-station --bin lumacore-server --features $features"
}
if ($builds["cli"]) { Cargo "lumacore-cli" "-p lumacore-cli" }
if ($builds["app"]) { Cargo "lumacore-app" "-p lumacore-app" }

# -- 3. Server ---------------------------------------------------------------
if ($runs["server"]) {
    Get-Process -Name "lumacore-server" -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Milliseconds 500
    $serverArgs = @()
    if (Test-Path $WebRoot) { $serverArgs += "--web-root", "`"$WebRoot`"" }
    else { Write-Warning "Web root $WebRoot not found - the server starts without the web UI" }
    if ($Show -ne "") { $serverArgs += "--show", $Show }
    if ($DebugBuild) { $serverArgs += "--debug" }
    if ($CleanStart) { $serverArgs += "--clean-start" }
    Write-Host "Starting $ServerBin $($serverArgs -join ' ')"
    Start-Process -FilePath $ServerBin -ArgumentList $serverArgs -WindowStyle Hidden `
        -RedirectStandardError $ServerLog
    $healthy = $false
    for ($i = 1; $i -le 30; $i++) {
        if (Server-Healthy) { $healthy = $true; break }
        Start-Sleep -Milliseconds 500
    }
    if (-not $healthy) { Fail "lumacore-server did not become healthy after 15 s (output: $ServerLog)" }
    Write-Host "Server is healthy. Web UI: http://127.0.0.1:$HttpPort/" -ForegroundColor Green
}

# -- 4. Windows --------------------------------------------------------------
if ($runs["tracy"]) {
    $machinePath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    $userPath    = [Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH    = "$machinePath;$userPath;$env:PATH"
    $tracyCmd = Get-Command tracy-profiler -ErrorAction SilentlyContinue
    if (-not $tracyCmd) { $tracyCmd = Get-Command tracy -ErrorAction SilentlyContinue }
    if (-not $tracyCmd) {
        $knownPath = "$env:LOCALAPPDATA\Programs\tracy\tracy-profiler.exe"
        if (Test-Path $knownPath) { $tracyCmd = Get-Item $knownPath }
    }
    if ($tracyCmd) {
        $tracyPath = if ($tracyCmd -is [System.IO.FileInfo]) { $tracyCmd.FullName } else { $tracyCmd.Source }
        Write-Host "Opening Tracy..."
        Start-Process -FilePath $tracyPath
    } else {
        Write-Host "Tracy not found - skipping (dev_init installs it; docs/dev/build/profiling.md)"
    }
}
if ($runs["meownitor"]) {
    Write-Host "Opening the Meownitor..."
    Start-Process -FilePath "cmd.exe" -ArgumentList "/c", "title MEOWNITOR && `"$CliBin`" --target 127.0.0.1 --meownitor"
}
if ($runs["web"]) {
    Write-Host "Opening the web UI: http://127.0.0.1:$HttpPort/"
    Start-Process "http://127.0.0.1:$HttpPort/"
}
if ($runs["app"]) {
    if (-not (Test-Path $ServerBin)) {
        Write-Host "Note: $ServerBin isn't built - the app reaches other stations, but can't start a local one"
    }
    Write-Host "Starting lumacore-app..."
    Start-Process -FilePath $AppBin
}
if ($runs["cli-window"]) {
    Write-Host "Opening the CLI (user $User)..."
    Start-Process -FilePath "cmd.exe" -ArgumentList "/c", "title LumaCore CLI && `"$CliBin`" --target 127.0.0.1 --user $User"
}

Write-Host "Plan done ($Profile)." -ForegroundColor Green

# -- 5. The CLI here, last ---------------------------------------------------
if ($runs["cli"]) {
    $cliArgs = @("--target", "127.0.0.1", "--user", $User)
    if ($Cmd -ne "") { $cliArgs += "--cmd", $Cmd }
    & $CliBin @cliArgs
}
Pop-Location
exit 0
