# scripts/pauli/openclaude/healthcheck.ps1
# Check whether the OpenClaude worker is installed and healthy on Windows.
#
# Exit codes:
#   0  - worker is healthy
#   1  - worker binary not found (install needed)
#   2  - worker binary found but not running (start needed)
#   3  - worker is running but gRPC port is not responding

param(
    [int]$Port = 0
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot   = (Resolve-Path (Join-Path $ScriptDir "..\..\..")).Path
$VendorDir  = Join-Path $RepoRoot "vendor\openclaude"
$HermesHome = if ($env:HERMES_HOME) { $env:HERMES_HOME } else { Join-Path $env:USERPROFILE ".hermes" }
$PidFile    = Join-Path $HermesHome "run\openclaude-worker.pid"

if ($Port -eq 0) { $Port = if ($env:OPENCLAUDE_PORT) { [int]$env:OPENCLAUDE_PORT } else { 50051 } }

function Log  { param([string]$Msg) Write-Host "[healthcheck] $Msg" }
function Warn { param([string]$Msg) Write-Warning "[healthcheck] WARN: $Msg" }
function Fail { param([string]$Msg) Write-Error "[healthcheck] FAIL: $Msg"; exit $args[0] }

# ---------------------------------------------------------------------------
# 1. Check binary
# ---------------------------------------------------------------------------
$BinPath = $null
$CB = Join-Path $VendorDir "bin\openclaude"
$CN = Join-Path $VendorDir "node_modules\.bin\openclaude.cmd"
if (Test-Path $CB)    { $BinPath = $CB }
elseif (Test-Path $CN) { $BinPath = $CN }
elseif (Get-Command "openclaude" -ErrorAction SilentlyContinue) { $BinPath = "openclaude" }

if (-not $BinPath) {
    Write-Error "[healthcheck] FAIL: openclaude binary not found. Run: .\scripts\pauli\openclaude\install.ps1"
    exit 1
}

$Version = & node $BinPath --version 2>&1 | Select-Object -First 1
Log "Binary:  $BinPath"
Log "Version: $Version"

# ---------------------------------------------------------------------------
# 2. Check process
# ---------------------------------------------------------------------------
if (-not (Test-Path $PidFile)) {
    Write-Warning "[healthcheck] No PID file at $PidFile. Worker may not be started."
    Write-Warning "Run: .\scripts\pauli\openclaude\start.ps1"
    exit 2
}

$PidContent = (Get-Content $PidFile -Raw).Trim()
if ($PidContent -eq "cli-stub") {
    Log "Mode:   cli-stub (per-task dispatch, no persistent process)"
    Log "Status: OK"
    exit 0
}

$WorkerPid = [int]$PidContent
$Proc = Get-Process -Id $WorkerPid -ErrorAction SilentlyContinue
if (-not $Proc) {
    Write-Error "[healthcheck] FAIL: Process $WorkerPid is not running. Run: .\scripts\pauli\openclaude\start.ps1"
    exit 2
}
Log "Process: running (PID $WorkerPid)"

# ---------------------------------------------------------------------------
# 3. Check gRPC port
# ---------------------------------------------------------------------------
try {
    $TcpClient = New-Object System.Net.Sockets.TcpClient
    $Connect   = $TcpClient.BeginConnect("localhost", $Port, $null, $null)
    $Success   = $Connect.AsyncWaitHandle.WaitOne(3000)
    if ($Success) {
        $TcpClient.EndConnect($Connect) | Out-Null
        Log "gRPC port ${Port}: responding"
    } else {
        Write-Warning "[healthcheck] gRPC port $Port not responding within 3s (may still be starting)"
        exit 3
    }
    $TcpClient.Close()
} catch {
    Write-Warning "[healthcheck] Could not probe gRPC port $Port: $_"
    exit 3
}

Log "Status: HEALTHY"
exit 0
