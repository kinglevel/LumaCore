# Stop any running lumacore-server processes (Windows).
#
# Usage:
#   .\scripts\stop_server.ps1

$procs = Get-Process -Name "lumacore-server" -ErrorAction SilentlyContinue

if (-not $procs) {
    Write-Host "No running lumacore-server found."
    exit 0
}

Write-Host "Stopping lumacore-server:"
foreach ($p in $procs) {
    Write-Host "  PID $($p.Id)"
}

$procs | Stop-Process -Force
Start-Sleep -Milliseconds 500

$remaining = Get-Process -Name "lumacore-server" -ErrorAction SilentlyContinue
if ($remaining) {
    Write-Warning "Some lumacore-server processes are still running."
    exit 1
}

Write-Host "lumacore-server stopped."
