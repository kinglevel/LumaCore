# Developer onboarding helper for LumaCore (Windows).
#
# What it does:
# - resolves the current LumaCore repo root
# - checks for basic developer tooling
# - reads .env.local for developer path variables
# - validates the configured paths
# - copies Obsidian CSS snippets
# - builds the lumacore-scripts launcher
#
# Usage:
#   .\scripts\win\dev_init.ps1

# Ensure cargo is in PATH (Rustup default install location)
$CargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "$env:USERPROFILE\.cargo" }
$CargoBin  = Join-Path $CargoHome "bin"
if ((Test-Path $CargoBin) -and ($env:PATH -notlike "*$CargoBin*")) {
    $env:PATH = "$CargoBin;$env:PATH"
}

$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot   = Split-Path -Parent (Split-Path -Parent $ScriptDir)
$EnvFile    = Join-Path $RepoRoot ".env.local"
$EnvExample = Join-Path $RepoRoot ".env.example"

function Say($msg) { Write-Host $msg }
function Warn($msg) { Write-Warning $msg }

Say "LumaCore developer init"
Say "  - repo root: $RepoRoot"

$missingTools = 0

Say ""
Say "Checking developer tools..."

foreach ($cmd in @("git", "rustc", "cargo")) {
    $found = Get-Command $cmd -ErrorAction SilentlyContinue
    if ($found) {
        Say "  - found ${cmd}: $($found.Source)"
    } else {
        Warn "missing $cmd"
        $missingTools = 1
    }
}

if ($missingTools -ne 0) {
    Say ""
    Warn "Some tools are missing. Install them before building."
    Warn "  Rust: winget install Rustlang.Rustup"
    Warn "  VS Build Tools: winget install Microsoft.VisualStudio.2022.BuildTools --override '--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'"
}

Say ""
Say "Checking environment configuration..."

$missingPaths = 0

if (Test-Path $EnvFile) {
    Say "  - found .env.local"

    foreach ($line in Get-Content $EnvFile) {
        $line = $line.Trim()
        if ($line -eq "" -or $line.StartsWith("#")) { continue }

        if ($line -notmatch "^([^=]+)=(.+)$") {
            Warn "invalid entry (expected NAME=path): $line"
            $missingPaths = 1
            continue
        }

        $name  = $Matches[1]
        $value = $Matches[2]

        # Resolve relative paths against repo root
        if (-not [System.IO.Path]::IsPathRooted($value)) {
            $value = Join-Path $RepoRoot $value
        }

        if (Test-Path $value) {
            Say "  - ok: ${name}=${value}"
        } else {
            Warn "missing path: ${name}=${value}"
            $missingPaths = 1
        }
    }
} else {
    Say "  - no .env.local found (optional)"
    Say "  - to configure companion repos, run:"
    Say "      copy .env.example .env.local"
    Say "      # then edit .env.local with your local paths"
}

Say ""
Say "Checking Tracy profiler..."

# tracy-client-sys 0.24.3 bundles Tracy v0.11.1 (protocol 69).
# The Tracy GUI MUST match this version exactly - newer versions cause
# "protocol mismatch" errors.
$TracyRequired = "0.11.1"

$tracyOk = $false
$tracyCmd = Get-Command tracy-profiler -ErrorAction SilentlyContinue
if (-not $tracyCmd) { $tracyCmd = Get-Command tracy -ErrorAction SilentlyContinue }

if ($tracyCmd) {
    # The prebuilt GUI binary does not support --version, so just check it exists.
    # Version is controlled by dev_init downloading the correct release.
    Say "  - Tracy profiler found: $($tracyCmd.Source)"
    $tracyOk = $true
} else {
    Say "  - Tracy profiler not found"
}

if (-not $tracyOk) {
    Say "  - Tracy v$TracyRequired is required (matches bundled tracy-client-sys)"
    Say "  - Do NOT use a package manager that ships a newer version."

    # Download prebuilt Windows binary from GitHub releases
    $TracyArchive = "$env:TEMP\tracy-windows-$TracyRequired.zip"
    $TracyExtract = "$env:TEMP\tracy-extract"
    $TracyUrl = "https://github.com/wolfpld/tracy/releases/download/v$TracyRequired/windows-$TracyRequired.zip"

    Say "  - Downloading Tracy v$TracyRequired from GitHub releases..."
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $TracyUrl -OutFile $TracyArchive -UseBasicParsing
    } catch {
        Warn "Download failed: $_"
        Say "  - Download manually from: $TracyUrl"
    }

    if (Test-Path $TracyArchive) {
        if (Test-Path $TracyExtract) { Remove-Item -Recurse -Force $TracyExtract }
        Expand-Archive -Path $TracyArchive -DestinationPath $TracyExtract -Force

        # Find Tracy.exe or tracy-profiler.exe in extracted files
        $TracyBin = Get-ChildItem -Path $TracyExtract -Recurse -Filter "Tracy.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
        if (-not $TracyBin) {
            $TracyBin = Get-ChildItem -Path $TracyExtract -Recurse -Filter "tracy-profiler.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
        }

        if ($TracyBin) {
            $InstallDir = "$env:LOCALAPPDATA\Programs\tracy"
            if (-not (Test-Path $InstallDir)) {
                New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
            }
            # Copy all files from the same directory (DLLs etc)
            $TracySrcDir = $TracyBin.DirectoryName
            Copy-Item -Path "$TracySrcDir\*" -Destination $InstallDir -Force -Recurse

            # Rename to tracy-profiler.exe if needed
            $installed = Join-Path $InstallDir "tracy-profiler.exe"
            if (-not (Test-Path $installed)) {
                $tracyExe = Join-Path $InstallDir "Tracy.exe"
                if (Test-Path $tracyExe) {
                    Copy-Item -Path $tracyExe -Destination $installed -Force
                }
            }

            # Add to user PATH if not already there
            $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
            if ($userPath -notlike "*$InstallDir*") {
                [Environment]::SetEnvironmentVariable("PATH", "$userPath;$InstallDir", "User")
                $env:PATH = "$env:PATH;$InstallDir"
                Say "  - Added $InstallDir to user PATH"
            }

            Say "  - Tracy profiler v$TracyRequired installed to $InstallDir"
        } else {
            Warn "Could not find Tracy executable in archive"
            Say "  - Check contents of $TracyExtract"
        }

        Remove-Item -Force $TracyArchive -ErrorAction SilentlyContinue
        Remove-Item -Recurse -Force $TracyExtract -ErrorAction SilentlyContinue
    }
}

Say ""
Say "Copying Obsidian snippets..."
$SnippetsSrc = Join-Path $RepoRoot "tools\obsidian-snippets"
$SnippetsDst = Join-Path $RepoRoot ".obsidian\snippets"
if (-not (Test-Path $SnippetsDst)) {
    New-Item -ItemType Directory -Path $SnippetsDst -Force | Out-Null
}
foreach ($snippet in Get-ChildItem -Path $SnippetsSrc -Filter "*.css" -ErrorAction SilentlyContinue) {
    $target = Join-Path $SnippetsDst $snippet.Name
    Copy-Item -Path $snippet.FullName -Destination $target -Force
    Say "  - copied: $($snippet.Name)"
}
Say "  Enable snippets in Obsidian: Settings > Appearance > CSS snippets"

Say ""
$scriptsRunning = Get-Process -Name "lumacore-scripts" -ErrorAction SilentlyContinue
if ($scriptsRunning) {
    Say "Skipping lumacore-scripts build (already running - cannot overwrite)."
} else {
    Say "Building lumacore-scripts launcher..."
    Push-Location $RepoRoot
    cmd /c "cargo build -p lumacore-scripts 2>&1"
    Pop-Location

    $scriptsBin = Join-Path $RepoRoot "target\debug\lumacore-scripts.exe"
    if (Test-Path $scriptsBin) {
        Say "  - built: $scriptsBin"
    } else {
        Warn "lumacore-scripts build failed"
    }
}

Say ""
Say "Developer onboarding complete."
Say "Next steps:"
Say "  1. copy .env.example .env.local  (if you haven't already)"
Say "  2. edit .env.local with your local companion repo paths"
Say "  3. cargo build"
Say "  4. cargo test --workspace"
Say "  5. .\scripts\win\start_server.ps1"

if ($missingPaths -ne 0) {
    exit 1
}
