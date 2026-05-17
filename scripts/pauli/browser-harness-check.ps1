$ErrorActionPreference = "Stop"

$cmd = Get-Command browser-harness -ErrorAction SilentlyContinue
if (-not $cmd) {
    throw "browser-harness was not found on PATH."
}

Write-Output "browser-harness available at $($cmd.Source)"
