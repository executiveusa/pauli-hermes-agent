# scripts/pauli/openclaude/start.ps1
# Start OpenClaude worker (Windows — gRPC mode via node background process).
#
# Usage:
#   .\scripts\pauli\openclaude\start.ps1 [-Port 50051] [-Mode grpc]
#
# Environment:
#   OPENCLAUDE_MODE   grpc|cli  (default: grpc)
#   OPENCLAUDE_PORT   gRPC port (default: 50051)

param(
    [string]$Mode = $null,
    [int]$Port    = 0
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot   = (Resolve-Path (Join-Path $ScriptDir "..\..\..")).Path
$VendorDir  = Join-Path $RepoRoot "vendor\openclaude"
$HermesHome = if ($env:HERMES_HOME) { $env:HERMES_HOME } else { Join-Path $env:USERPROFILE ".hermes" }
$LogFile    = if ($env:OPENCLAUDE_LOG) { $env:OPENCLAUDE_LOG } else { Join-Path $HermesHome "logs\openclaude-worker.log" }
$PidFile    = Join-Path $HermesHome "run\openclaude-worker.pid"

if (-not $Mode) { $Mode = if ($env:OPENCLAUDE_MODE) { $env:OPENCLAUDE_MODE } else { "grpc" } }
if ($Port -eq 0) { $Port = if ($env:OPENCLAUDE_PORT) { [int]$env:OPENCLAUDE_PORT } else { 50051 } }

function Log  { param([string]$Msg) Write-Host "[start.ps1] $Msg" }
function Fail { param([string]$Msg) Write-Error "[start.ps1] ERROR: $Msg"; exit 1 }

# ---------------------------------------------------------------------------
# Find binary
# ---------------------------------------------------------------------------
$BinPath = $null
$CandidateBin = Join-Path $VendorDir "bin\openclaude"
$CandidateNm  = Join-Path $VendorDir "node_modules\.bin\openclaude.cmd"

if (Test-Path $CandidateBin)    { $BinPath = $CandidateBin }
elseif (Test-Path $CandidateNm) { $BinPath = $CandidateNm }
elseif (Get-Command "openclaude" -ErrorAction SilentlyContinue) { $BinPath = "openclaude" }
else { Fail "openclaude binary not found. Run: .\scripts\pauli\openclaude\install.ps1" }

# ---------------------------------------------------------------------------
# Check if already running
# ---------------------------------------------------------------------------
if (Test-Path $PidFile) {
    $OldPid = Get-Content $PidFile -Raw
    if ($OldPid -eq "cli-stub") {
        Log "Worker is in cli-stub mode. No restart needed."
        exit 0
    }
    $OldPidInt = [int]$OldPid.Trim()
    $Proc = Get-Process -Id $OldPidInt -ErrorAction SilentlyContinue
    if ($Proc) {
        Log "OpenClaude worker already running (PID $OldPidInt). Use Stop-Process to stop it first."
        exit 0
    } else {
        Log "Stale PID file found (PID $OldPidInt is dead). Cleaning up."
        Remove-Item $PidFile -Force
    }
}

# ---------------------------------------------------------------------------
# Ensure directories exist
# ---------------------------------------------------------------------------
$LogDir = Split-Path $LogFile -Parent
$PidDir = Split-Path $PidFile -Parent
if (-not (Test-Path $LogDir)) { New-Item -ItemType Directory -Path $LogDir -Force | Out-Null }
if (-not (Test-Path $PidDir)) { New-Item -ItemType Directory -Path $PidDir -Force | Out-Null }

# ---------------------------------------------------------------------------
# Start worker process
# ---------------------------------------------------------------------------
if ($Mode -eq "grpc") {
    Log "Starting OpenClaude worker (mode=grpc, port=$Port)"
    Log "Log: $LogFile"

    $NodeArgs = @($BinPath, "--headless", "--grpc-port", "$Port")
    $Process  = Start-Process -FilePath "node" -ArgumentList $NodeArgs `
                              -RedirectStandardOutput $LogFile `
                              -RedirectStandardError  $LogFile `
                              -WindowStyle Hidden -PassThru

    $Process.Id | Out-File -FilePath $PidFile -Encoding ascii
    Start-Sleep -Seconds 2

    if ($Process.HasExited) {
        Fail "Worker exited immediately. Check log: $LogFile"
    }

    Log "Worker started with PID $($Process.Id)"
} else {
    Log "CLI mode: each task dispatched as a separate process (no persistent worker)"
    "cli-stub" | Out-File -FilePath $PidFile -Encoding ascii
}

Log "Healthcheck: .\scripts\pauli\openclaude\healthcheck.ps1"
