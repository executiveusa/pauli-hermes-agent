# Customization

## The Operating System

The `config/` directory is the agent's brain — it defines who the agent is and how it behaves.

| File | Purpose |
|------|---------|
| `SOUL.md` | Agent identity, personality traits, and values |
| `EVENT_HANDLER.md` | Event handler system prompt |
| `JOB_SUMMARY.md` | Prompt for summarizing completed jobs |
| `HEARTBEAT.md` | Self-monitoring behavior |
| `AGENT.md` | Agent runtime environment |
| `CRONS.json` | Scheduled job definitions |
| `TRIGGERS.json` | Webhook trigger definitions |

Each job automatically gets its own `logs/<JOB_ID>/job.md` file created by the event handler. Jobs are created via Telegram chat, webhooks, or cron schedules.

---

## Using Your Bot

There are several ways to interact with your agent — web chat, Telegram, webhooks, and scheduled jobs. See [Chat Integrations](CHAT_INTEGRATIONS.md) for details on adding other channels.

### Web Chat

Visit your APP_URL to access the built-in web chat interface. Features include:

- **Streaming responses** — AI responses stream in real-time
- **File uploads** — Send images, PDFs, and text files
- **Chat history** — Browse and resume past conversations
- **Job management** — Create and monitor agent jobs from the Swarm page

The web chat is available out of the box after setup — no additional configuration needed.

### Telegram Chat (Optional)

Connect a Telegram bot to chat with your agent on the go. Set up with:

```bash
npm run setup-telegram
```

The bot uses your LLM to understand requests and can:

- **Chat** — Have a conversation, ask questions, get information
- **Create jobs** — Say "create a job to..." and the bot will spawn an autonomous agent

**Security:** During setup, you'll verify your chat ID. Once configured, the bot only responds to messages from your authorized chat and ignores everyone else.

#### Voice Messages

Send voice notes to your bot and they'll be transcribed using OpenAI Whisper.

**Requirements:**
- `OPENAI_API_KEY` in your `.env` file

The bot automatically detects voice messages and transcribes them before processing.

### Webhooks

Create jobs programmatically via HTTP:

```bash
curl -X POST https://your-app-url/api/create-job \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{"job": "Update the README with installation instructions"}'
```

API keys are managed via the web UI Settings page.

### Scheduled Jobs

Define recurring jobs in `config/CRONS.json`:

```json
[
  {
    "name": "daily-check",
    "schedule": "0 9 * * *",
    "job": "Check for dependency updates",
    "enabled": true
  }
]
```

Set `"enabled": true` to activate a scheduled job.

---

## Skills

Add custom skills for the agent in `.pi/skills/`. Skills extend the agent's capabilities with specialized tools and behaviors.

---

## Security

| What the AI tries | What happens |
|-------------------|--------------|
| `echo $ANTHROPIC_API_KEY` | Empty |
| `echo $GH_TOKEN` | Empty |
| `cat /proc/self/environ` | Secrets missing |
| Claude API calls | Work normally |
| GitHub CLI commands | Work normally |

### How Secret Protection Works

1. GitHub passes a single `SECRETS` env var (base64-encoded JSON with all credentials)
2. The entrypoint decodes the JSON and exports each key as an env var
3. Pi starts - SDKs read their env vars (ANTHROPIC_API_KEY, gh CLI uses GH_TOKEN)
4. The `env-sanitizer` extension filters ALL secret keys from bash subprocess env
5. The LLM can't `echo $ANYTHING` - subprocess env is filtered
6. Other extensions still have full `process.env` access

**What's Protected:**

Any key in the `SECRETS` JSON is automatically filtered from the LLM's bash environment. The `SECRETS` variable itself is also filtered.

```bash
# If your SECRETS contains:
{"GH_TOKEN": "...", "ANTHROPIC_API_KEY": "...", "MY_CUSTOM_KEY": "..."}

# Then all of these return empty:
echo $GH_TOKEN           # empty
echo $ANTHROPIC_API_KEY  # empty
echo $MY_CUSTOM_KEY      # empty
```

### LLM-Accessible Secrets

Sometimes you want the LLM to have access to certain credentials - browser logins, skill API keys, or service passwords. Use `LLM_SECRETS` for these.

```bash
# Protected (filtered from LLM)
SECRETS=$(echo -n '{"GH_TOKEN":"ghp_xxx","ANTHROPIC_API_KEY":"sk-ant-xxx"}' | base64)

# Accessible to LLM (not filtered)
LLM_SECRETS=$(echo -n '{"BROWSER_PASSWORD":"mypass123"}' | base64)
```

| Credential Type | Put In | Why |
|-----------------|--------|-----|
| `GH_TOKEN` | `SECRETS` | Agent shouldn't push to arbitrary repos |
| `ANTHROPIC_API_KEY` | `SECRETS` | Agent shouldn't leak billing keys |
| Browser login password | `LLM_SECRETS` | Skills may need to authenticate |
| Third-party API key for a skill | `LLM_SECRETS` | Skills need these to function |

### Implementation

The `env-sanitizer` extension in `.pi/extensions/` dynamically filters secrets:

```typescript
const bashTool = createBashTool(process.cwd(), {
  spawnHook: ({ command, cwd, env }) => {
    const filteredEnv = { ...env };
    if (process.env.SECRETS) {
      try {
        for (const key of Object.keys(JSON.parse(process.env.SECRETS))) {
          delete filteredEnv[key];
        }
      } catch {}
    }
    delete filteredEnv.SECRETS;
    return { command, cwd, env: filteredEnv };
  },
});
```

No special Docker flags required. Works on any host.

### Custom Extensions

The env-sanitizer protects against the **AI agent** accessing secrets through bash. Extension code itself can access `process.env` directly - this is by design.

**Best practices:**
- Don't create tools that echo environment variables to the agent
- Review extension code before adding to your agent
