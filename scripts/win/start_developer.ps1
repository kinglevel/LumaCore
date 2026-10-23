# Developer environment: build the server (+ Tracy) and the CLI, restart the
# server with the show (demo unless -Show), then open Tracy (if installed),
# the Meownitor and the CLI in their own windows. A preset of dev.ps1.
#
# Usage:
#   .\scripts\win\start_developer.ps1 [-Show demo] [-User Admin] [-DebugBuild] [-CleanStart]

param(
    [string]$Show = "demo",
    [string]$User = "Admin",
    [switch]$DebugBuild,
    [switch]$CleanStart
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
& (Join-Path $ScriptDir "dev.ps1") -Build "server,cli" -Tracy -Run "server,tracy,meownitor,cli-window" `
    -Show $Show -User $User -DebugBuild:$DebugBuild -CleanStart:$CleanStart
exit $LASTEXITCODE
