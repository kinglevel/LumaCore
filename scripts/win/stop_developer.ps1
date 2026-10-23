# Stop all processes the dev plans start (server, CLI, Meownitor, Tracy, app).
# Does NOT kill lumacore-scripts (the TUI launcher).
#
# Usage:
#   .\scripts\win\stop_developer.ps1

$targets = @("lumacore-server", "lumacore-cli", "lumacore-app", "tracy-profiler", "tracy-capture", "Tracy")

$found = @()
foreach ($name in $targets) {
    $procs = Get-Process -Name $name -ErrorAction SilentlyContinue
    if ($procs) {
        $found += $procs
    }
}

if ($found.Count -eq 0) {
    Write-Host "No running developer processes found."
    exit 0
}

Write-Host "Stopping developer processes:"
foreach ($p in $found) {
    Write-Host "  $($p.ProcessName) (PID $($p.Id))"
}

$found | Stop-Process -Force
Start-Sleep -Milliseconds 500

# Verify
$remaining = @()
foreach ($name in $targets) {
    $procs = Get-Process -Name $name -ErrorAction SilentlyContinue
    if ($procs) {
        $remaining += $procs
    }
}

if ($remaining.Count -gt 0) {
    Write-Warning "Some developer processes are still running:"
    foreach ($p in $remaining) {
        Write-Warning "  $($p.ProcessName) (PID $($p.Id))"
    }
    exit 1
}

Write-Host "All developer processes stopped."
