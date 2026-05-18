# scripts/pauli/openclaude/install.ps1
# Clone or update vendor/openclaude, install npm deps, and verify the binary.
#
# Usage:
#   .\scripts\pauli\openclaude\install.ps1 [-Force]
#
# Options:
#   -Force   Re-clone even if vendor/openclaude already exists.
#
# Environment variables:
#   OPENCLAUDE_CLONE_URL  Override clone URL  (default: GitHub upstream)
#   OPENCLAUDE_CLONE_TAG  Branch/tag to clone (default: main)

param(
    [switch]$Force = $false
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot  = (Resolve-Path (Join-Path $ScriptDir "..\..\..")).Path
$VendorDir = Join-Path $RepoRoot "vendor\openclaude"
$CloneUrl  = if ($env:OPENCLAUDE_CLONE_URL) { $env:OPENCLAUDE_CLONE_URL } else { "https://github.com/Gitlawb/openclaude.git" }
$CloneTag  = if ($env:OPENCLAUDE_CLONE_TAG) { $env:OPENCLAUDE_CLONE_TAG } else { "main" }

function Log   { param([string]$Msg) Write-Host "[install.ps1] $Msg" }
function Fail  { param([string]$Msg) Write-Error "[install.ps1] ERROR: $Msg"; exit 1 }
function Need  {
    param([string]$Cmd)
    if (-not (Get-Command $Cmd -ErrorAction SilentlyContinue)) {
        Fail "$Cmd is not installed. Please install it and try again."
    }
}

# ---------------------------------------------------------------------------
# Prerequisite checks
# ---------------------------------------------------------------------------
Need git
Need node
Need npm

$NodeVersion = (node --version).TrimStart('v')
$NodeMajor   = [int]($NodeVersion.Split('.')[0])
if ($NodeMajor -lt 22) {
    Fail "Node.js >= 22 required. Found: v$NodeVersion. Install via: nvm install 22"
}
Log "Node.js v$NodeVersion OK"

# ---------------------------------------------------------------------------
# Clone or update
# ---------------------------------------------------------------------------
if ((Test-Path $VendorDir) -and -not $Force) {
    Log "vendor/openclaude already exists — pulling latest $CloneTag"
    Push-Location $VendorDir
    try {
        git fetch origin
        git checkout $CloneTag
        git pull --ff-only origin $CloneTag
    } catch {
        Log "fast-forward failed; doing a hard reset"
        git reset --hard "origin/$CloneTag"
    } finally {
        Pop-Location
    }
} else {
    if ((Test-Path $VendorDir) -and $Force) {
        Log "-Force: removing existing vendor/openclaude"
        Remove-Item -Recurse -Force $VendorDir
    }
    Log "Cloning $CloneUrl@$CloneTag -> vendor/openclaude"
    git clone --depth=1 --branch $CloneTag $CloneUrl $VendorDir
}

# ---------------------------------------------------------------------------
# Install npm dependencies
# ---------------------------------------------------------------------------
Log "Running npm install in vendor/openclaude..."
Push-Location $VendorDir
try {
    npm install --prefer-offline 2>&1 | Select-Object -Last 5 | ForEach-Object { Log $_ }
} finally {
    Pop-Location
}

# ---------------------------------------------------------------------------
# Build (if build script exists)
# ---------------------------------------------------------------------------
Push-Location $VendorDir
try {
    $BuildOutput = npm run build 2>&1 | Select-Object -Last 5
    $BuildOutput | ForEach-Object { Log $_ }
    Log "Build OK"
} catch {
    Log "Build step skipped or not required"
} finally {
    Pop-Location
}

# ---------------------------------------------------------------------------
# Verify binary
# ---------------------------------------------------------------------------
$BinPath = Join-Path $VendorDir "bin\openclaude"
if (-not (Test-Path $BinPath)) {
    $BinPath = Join-Path $VendorDir "node_modules\.bin\openclaude.cmd"
    if (-not (Test-Path $BinPath)) {
        $BinPath = Join-Path $VendorDir "node_modules\.bin\openclaude"
        if (-not (Test-Path $BinPath)) {
            Fail "openclaude binary not found at $VendorDir\bin\openclaude or node_modules\.bin\"
        }
    }
}

$Version = & node $BinPath --version 2>&1 | Select-Object -First 1
Log "Installed: $Version"
Log "Binary: $BinPath"
Log ""
Log "Installation complete. Next steps:"
Log "  1. Generate config:  .\scripts\pauli\openclaude\generate-config.ps1"
Log "  2. Start worker:     .\scripts\pauli\openclaude\start.ps1"
Log "  3. Health check:     .\scripts\pauli\openclaude\healthcheck.ps1"
