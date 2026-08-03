# 21st.dev setup on Windows and client machines

## Scope

This guide connects the 21st CLI and optional MCP endpoint without storing credentials in Git. Use it on the owner's Surface or a client-owned Windows computer only with visible consent.

## Read-only inspection

Run in PowerShell:

```powershell
node --version
npm --version
Get-Command 21st -ErrorAction SilentlyContinue
python skills/21st-dev-mcp/scripts/doctor.py --project .
```

Stop if Node or npm is missing. Use the machine owner's approved Node.js LTS installation method; do not silently install a second Node distribution.

## Install CLI

```powershell
npm i -g @21st-dev/cli
21st --version
```

Record the version. Do not run with administrator privileges unless the existing Node installation genuinely requires it and the owner approves the prompt.

## Authenticate

```powershell
21st login
21st whoami
```

The login command opens a browser. The owner completes authentication directly. Hermes must not ask the owner to paste the token into chat or capture the browser credentials.

## Optional environment variable for scripts

Set `API_KEY_21ST` through the owner's approved secret mechanism. Do not use `setx` by default because it can create persistent plaintext environment data visible to other processes and future support sessions.

For one current PowerShell session, the owner may set it directly without displaying it afterward:

```powershell
$env:API_KEY_21ST = Read-Host "Enter 21st API key"
```

Do not echo the variable. Clear the session value after the work when requested:

```powershell
Remove-Item Env:API_KEY_21ST
```

## Configure an MCP client

Prefer the official initializer for supported clients:

```powershell
21st init --client claude
21st init --client cursor
21st init --client codex
```

Run only the command for the intended client. Before and after the command:

1. back up the existing MCP configuration;
2. inspect the diff;
3. verify existing MCP servers remain present;
4. confirm the new endpoint is `https://21st.dev/api/mcp`;
5. confirm configuration references `API_KEY_21ST` instead of containing a literal `21st_sk_` key.

## Hermes CLI-first mode

Hermes can use the CLI without a native MCP client:

```powershell
21st search "accessible pricing table"
21st get <component-reference>
```

Installation is a state-changing action:

```powershell
21st add <component-reference>
```

Before `add`, record the Git SHA, package lockfile, planned files, dependency impact, and rollback. Obtain owner approval.

## Connection verification

Run:

```powershell
python skills/21st-dev-mcp/scripts/doctor.py --project .
```

The doctor may report `21ST_MCP_CONFIGURED`. This is not proof that the MCP server works. From the intended client, perform one authenticated tool-list or search request. Only then record `21ST_MCP_VERIFIED`.

## Client or friend's laptop

Use the following loop for every state-changing action:

```text
OBSERVE -> EXPLAIN -> APPROVE -> ACT -> VERIFY -> RECORD
```

Requirements:

- device owner is present;
- scope and target project are recorded;
- owner enters account and administrator credentials;
- no persistent remote account or hidden access is created;
- API key stays on the owner's device;
- remote support session is disconnected after verification;
- provide uninstall and logout instructions.

## Uninstall and rollback

```powershell
npm uninstall -g @21st-dev/cli
```

Remove only the 21st server entry from the MCP configuration. Restore the backup if the initializer affected unrelated entries. Clear `API_KEY_21ST` when the owner requests disconnection. Re-run the MCP client's existing server checks to confirm other integrations remain functional.
