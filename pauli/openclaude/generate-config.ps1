# scripts/pauli/openclaude/generate-config.ps1
# Generate ~/.openclaude.json from environment variables on Windows.
#
# SECURITY: Reads API keys from environment variables only.
# ~/.openclaude.json is in .gitignore and MUST NEVER be committed.
# Source API keys from Infisical, dotenv, or Windows credential store.
#
# Usage:
#   .\scripts\pauli\openclaude\generate-config.ps1 [-Provider openrouter] [-DryRun]

param(
    [string]$Provider = "",
    [switch]$DryRun   = $false,
    [string]$Output   = ""
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$OutputFile = if ($Output) { $Output } else { Join-Path $env:USERPROFILE ".openclaude.json" }

function Log  { param([string]$Msg) Write-Host "[generate-config] $Msg" }
function Fail { param([string]$Msg) Write-Error "[generate-config] ERROR: $Msg"; exit 1 }
function Warn { param([string]$Msg) Write-Warning "[generate-config] WARN: $Msg" }
function Env  { param([string]$Name) [System.Environment]::GetEnvironmentVariable($Name) }

# ---------------------------------------------------------------------------
# Auto-select provider (cheapest first)
# ---------------------------------------------------------------------------
if (-not $Provider) {
    $OllamaHost = Env "OLLAMA_HOST"
    $HasOllama  = $OllamaHost -or (Get-Command "ollama" -ErrorAction SilentlyContinue)
    if ($HasOllama)                   { $Provider = "ollama"      }
    elseif (Env "OPENROUTER_API_KEY") { $Provider = "openrouter" }
    elseif (Env "GROQ_API_KEY")       { $Provider = "groq"        }
    elseif (Env "DEEPSEEK_API_KEY")   { $Provider = "deepseek"    }
    elseif (Env "OPENAI_API_KEY")     { $Provider = "openai"      }
    else {
        Fail "No provider key found in environment. Set OPENROUTER_API_KEY, GROQ_API_KEY, DEEPSEEK_API_KEY, OPENAI_API_KEY, or OLLAMA_HOST."
    }
}

Log "Selected provider: $Provider"

# ---------------------------------------------------------------------------
# Build config per provider
# ---------------------------------------------------------------------------
$Config = $null
switch ($Provider) {
    "ollama" {
        $OllamaEndpoint = if (Env "OLLAMA_HOST") { (Env "OLLAMA_HOST").TrimEnd("/") + "/v1" } else { "http://localhost:11434/v1" }
        $Model = if (Env "OPENAI_MODEL") { Env "OPENAI_MODEL" } else { "qwen2.5-coder:7b" }
        $Config = [ordered]@{ provider = "ollama"; apiKey = "ollama"; model = $Model; baseUrl = $OllamaEndpoint }
    }
    "openrouter" {
        $Key = Env "OPENROUTER_API_KEY"
        if (-not $Key) { Fail "OPENROUTER_API_KEY is not set" }
        $Model = if (Env "OPENAI_MODEL") { Env "OPENAI_MODEL" } else { "meta-llama/llama-3.1-8b-instruct:free" }
        $Config = [ordered]@{ provider = "openrouter"; apiKey = $Key; model = $Model; baseUrl = "https://openrouter.ai/api/v1" }
    }
    "groq" {
        $Key = Env "GROQ_API_KEY"
        if (-not $Key) { Fail "GROQ_API_KEY is not set" }
        $Model = if (Env "OPENAI_MODEL") { Env "OPENAI_MODEL" } else { "llama-3.1-8b-instant" }
        $Config = [ordered]@{ provider = "groq"; apiKey = $Key; model = $Model; baseUrl = "https://api.groq.com/openai/v1" }
    }
    "deepseek" {
        $Key = Env "DEEPSEEK_API_KEY"
        if (-not $Key) { Fail "DEEPSEEK_API_KEY is not set" }
        $Model = if (Env "OPENAI_MODEL") { Env "OPENAI_MODEL" } else { "deepseek-coder" }
        $Config = [ordered]@{ provider = "deepseek"; apiKey = $Key; model = $Model; baseUrl = "https://api.deepseek.com/v1" }
    }
    "openai" {
        $Key = Env "OPENAI_API_KEY"
        if (-not $Key) { Fail "OPENAI_API_KEY is not set" }
        $Model   = if (Env "OPENAI_MODEL")   { Env "OPENAI_MODEL"   } else { "gpt-4o-mini" }
        $BaseUrl = if (Env "OPENAI_BASE_URL") { Env "OPENAI_BASE_URL" } else { "https://api.openai.com/v1" }
        $Config = [ordered]@{ provider = "openai"; apiKey = $Key; model = $Model; baseUrl = $BaseUrl }
    }
    default { Fail "Unknown provider: $Provider. Supported: ollama, openrouter, groq, deepseek, openai" }
}

$JsonOutput = $Config | ConvertTo-Json -Depth 5

# ---------------------------------------------------------------------------
# Write or preview
# ---------------------------------------------------------------------------
if ($DryRun) {
    Log "DRY RUN — would write to $OutputFile:"
    $Redacted = $JsonOutput -replace '"apiKey": "[^"]*"', '"apiKey": "***REDACTED***"'
    Write-Host $Redacted
} else {
    # Write with restrictive ACL (current user only)
    $JsonOutput | Out-File -FilePath $OutputFile -Encoding UTF8 -Force

    # Restrict to current user on Windows
    try {
        $Acl = Get-Acl $OutputFile
        $Acl.SetAccessRuleProtection($true, $false)
        $Rule = New-Object System.Security.AccessControl.FileSystemAccessRule(
            [System.Security.Principal.WindowsIdentity]::GetCurrent().Name,
            "FullControl", "Allow"
        )
        $Acl.SetAccessRule($Rule)
        Set-Acl -Path $OutputFile -AclObject $Acl
    } catch {
        Warn "Could not set file ACL — manually restrict $OutputFile to your user only."
    }

    Log "Config written to $OutputFile"
    Log "Key value is NOT shown — verify the file manually if needed."
}

Log ""
Log "IMPORTANT: $OutputFile contains secrets. It is in .gitignore and must NEVER be committed."
