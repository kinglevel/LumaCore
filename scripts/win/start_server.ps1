# Build, (re)start lumacore-server in the background, then attach the CLI here
# (or run one -Cmd). A preset of dev.ps1.
#
# Usage:
#   .\scripts\win\start_server.ps1 [-Show demo] [-User LD] [-Cmd "list fixture"] [-DebugBuild] [-CleanStart]

param(
    [switch]$DebugBuild,
    [string]$Show = "",
    [string]$User = "Admin",
    [string]$Cmd  = "",
    [switch]$CleanStart
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
& (Join-Path $ScriptDir "dev.ps1") -Build "server,cli" -Run "server,cli" `
    -Show $Show -User $User -Cmd $Cmd -DebugBuild:$DebugBuild -CleanStart:$CleanStart
exit $LASTEXITCODE
