# Hermes WebUI Setup for Pauli Agent

The Hermes WebUI has been successfully installed and configured to connect to your Pauli Agent!

## Running Status

✅ **WebUI Server**: Running on `http://localhost:3000`
✅ **Agent API**: Connected to `http://localhost:8642/v1` (OpenAI-compatible API)

## Features Enabled

- ✅ Chat interface with streaming responses
- ✅ Session management (save/load conversations)
- ✅ File browser and workspace management
- ✅ Profile and model switching
- ✅ Settings and configuration
- ✅ Remote agent mode (connecting to your Pauli Agent API)

## Configuration

The WebUI is configured via environment variables in `/webui/.env`:

```env
HERMES_WEBUI_HOST=0.0.0.0           # Listen on all interfaces
HERMES_WEBUI_PORT=3000              # Web server port
HERMES_WEBUI_BOT_NAME="Pauli Agent" # Display name in UI
HERMES_AGENT_API_BASE=http://localhost:8642/v1  # Agent API endpoint
HERMES_REMOTE_AGENT=true            # Enable remote agent mode
```

## How to Access

1. **In Preview**: The v0 preview should automatically detect port 3000 and show the Hermes WebUI
2. **Direct URL**: `http://localhost:3000`
3. **Remote**: If deployed, use the public URL from your Vercel project

## Directory Structure

- `/webui/` - Hermes WebUI application
- `/webui/.env` - Environment configuration
- `~/.hermes/` - User state directory (sessions, credentials, etc.)
- `~/workspace/` - Default workspace for file operations

## Important Notes

⚠️ **Security**: The WebUI is currently running without authentication on `0.0.0.0`. 
- For production, set `HERMES_WEBUI_PASSWORD` or bind to `127.0.0.1`
- See the warning in the startup logs

## Commands

**Start the WebUI**:
```bash
cd /vercel/share/v0-project/webui
source ../webui_env/bin/activate
set -a && source .env && set +a
python server.py
```

**View logs**:
```bash
tail -f /tmp/webui_direct.log
```

## Customization

To customize the WebUI for your agent:

1. **Change bot name**: Update `HERMES_WEBUI_BOT_NAME` in `.env`
2. **Add authentication**: Set `HERMES_WEBUI_PASSWORD` in `.env`
3. **Change API endpoint**: Update `HERMES_AGENT_API_BASE` to your Pauli Agent API
4. **Configure workspace**: Update `HERMES_WEBUI_DEFAULT_WORKSPACE` path

## Next Steps

1. ✅ WebUI is running and accessible
2. Open the preview to interact with your agent via the Hermes WebUI
3. Create conversations, browse files, and switch models as needed
4. Configure additional settings through the Settings menu in the WebUI

---

For more information, visit: https://github.com/nesquena/hermes-webui
