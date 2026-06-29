param(
    [string]$EnvFile = "E:\THE PAULI FILES\master.env",
    [string]$HermesHome = "$HOME\.hermes",
    [string]$ProfileName = "pauli"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $EnvFile)) {
    throw "Env file not found: $EnvFile"
}

$hermesHomeResolved = [System.IO.Path]::GetFullPath($HermesHome)
$profileDir = Join-Path $hermesHomeResolved "profiles\$ProfileName"
$profileEnv = Join-Path $profileDir ".env"
$rootEnv = Join-Path $hermesHomeResolved ".env"

New-Item -ItemType Directory -Force -Path $hermesHomeResolved | Out-Null
New-Item -ItemType Directory -Force -Path $profileDir | Out-Null

$content = Get-Content -LiteralPath $EnvFile -Raw
$normalized = $content -replace "`r`n", "`n"
$required = @(
    "API_SERVER_ENABLED=true",
    "API_SERVER_CORS_ORIGINS=*"
)

foreach ($line in $required) {
    $key = ($line -split "=", 2)[0]
    if ($normalized -notmatch "(?m)^$([regex]::Escape($key))=") {
        $normalized = $normalized.TrimEnd() + "`n" + $line + "`n"
    }
}

Set-Content -LiteralPath $profileEnv -Value $normalized -NoNewline
Set-Content -LiteralPath $rootEnv -Value $normalized -NoNewline

Write-Output "Synced redacted env source into $profileEnv and $rootEnv"
