# Agent Reach setup for Hermes

## Installation model

Keep Agent Reach as an overlay. Do not vendor its source into Hermes and do not
add it as a mandatory Hermes core dependency. The bundled Hermes skill teaches
the agent when and how to call Agent Reach; the CLI and upstream tools live in
the user's home environment.

Pinned integration baseline:

- Agent Reach: `1.5.0`
- Upstream commit: `b4d52c46c9113cb0f653d6df4cf71ebadf4930ac`
- Python: `3.10+`

## Linux/macOS bootstrap

From a loaded Hermes skill, resolve the injected skill directory and run:

```bash
bash "<skill-directory>/scripts/bootstrap.sh" --apply
```

Preview without changing the environment:

```bash
bash "<skill-directory>/scripts/bootstrap.sh" --check
```

Install optional channels only when the user names them:

```bash
bash "<skill-directory>/scripts/bootstrap.sh" --apply --channels opencli,twitter
```

The script:

1. verifies Python 3.10+;
2. installs the pinned Agent Reach build with `pipx`, or a dedicated venv when
   `pipx` is unavailable;
3. never runs `sudo`;
4. invokes Agent Reach's own installer;
5. runs `agent-reach doctor --json`.

If system packages are required, stop and report the exact missing package and
command. Do not elevate permissions automatically.

## Windows

Use PowerShell and the Python Launcher:

```powershell
py -3 -m venv $env:USERPROFILE\.agent-reach-venv
$env:USERPROFILE\.agent-reach-venv\Scripts\Activate.ps1
python -m pip install "https://github.com/Panniantong/Agent-Reach/archive/b4d52c46c9113cb0f653d6df4cf71ebadf4930ac.zip"
agent-reach install --env=auto
agent-reach doctor --json
```

Do not use the Microsoft Store `python3.exe` alias.

## Health gate

Run before login-backed or multi-backend tasks:

```bash
agent-reach doctor --json
```

Record:

- platform/channel;
- status;
- `active_backend`;
- missing dependency or credential;
- whether the failure is local, network, authentication, or platform blocking.

Do not silently switch to an undocumented command. Read the matching reference
and follow its retry order.

## Base zero-config channels

After a normal install, expect these to be available without account cookies:

- web pages through Jina Reader;
- YouTube metadata, search, and subtitles through `yt-dlp`;
- RSS/Atom through `feedparser`;
- public GitHub through `gh`;
- Exa search through `mcporter` when configured by the installer;
- V2EX public API;
- basic Bilibili search/details through `bili-cli`.

## Credential boundaries

Use a dedicated/secondary account for cookie-backed platforms. Browser cookies
and tokens can grant full account access.

- Never read arbitrary browser cookies.
- Never automate login.
- Never put credentials in a repo, prompt, transcript, log, or command output.
- Pass required values only in the child process environment.
- Redact doctor output before sharing if it contains sensitive path or account
  information.

## Updates

Check after substantial research runs, not before or during them:

```bash
agent-reach check-update
```

Upgrade only as a separate bounded change:

```bash
pipx upgrade agent-reach
agent-reach doctor --json
```

For this Hermes integration, update the pinned commit and `UPSTREAM.md` only
after the new version passes the transcript and doctor smoke checks.
