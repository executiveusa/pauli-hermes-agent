# Hostinger MCP Server Setup

Configure Claude Desktop to directly manage your Hostinger VPS via Model Context Protocol (MCP).

## Setup Instructions

### 1. Find Your Claude Desktop Config

**macOS:**
```
~/Library/Application Support/Claude/claude_desktop_config.json
```

**Windows:**
```
%APPDATA%\Claude\claude_desktop_config.json
```

**Linux:**
```
~/.config/Claude/claude_desktop_config.json
```

### 2. Add Hostinger MCP Server

Open your `claude_desktop_config.json` and add this to the `mcpServers` section:

```json
{
  "mcpServers": {
    "hostinger-mcp": {
      "type": "stdio",
      "command": "npx",
      "args": [
        "hostinger-api-mcp@latest"
      ],
      "env": {
        "API_TOKEN": "${HOSTINGER_API_KEY}"
      }
    },
    "hermes-rolodex": {
      "type": "stdio",
      "command": "python3",
      "args": [
        "/opt/pauli-hermes-agent/mcp-servers/hermes-rolodex/server.py"
      ],
      "env": {
        "HERMES_CONFIG": "~/.hermes/config.yaml"
      }
    }
  }
}
```

### 3. Restart Claude Desktop

Close and reopen Claude Desktop to load the MCP servers.

### 4. Verify Connection

In Claude, you should now see:
- 🔌 **hostinger-mcp** — Hostinger API tools
- 🧠 **hermes-rolodex** — Hermes agent tools

---

## What You Can Do With Hostinger MCP

Once connected, you can use Claude to:

```
"Deploy my Hermes Agent to my VPS on Hostinger"
→ Claude uses MCP to configure everything automatically

"Show me my VPS instances"
→ Lists all VPS instances from your Hostinger account

"Reboot my production VPS"
→ Sends reboot command via Hostinger API

"Add DNS records for pauli-hermes-agent.com"
→ Creates DNS entries automatically

"Check my VPS billing"
→ Retrieves account and billing information
```

---

## Complete Setup: Combine Everything

Your full `claude_desktop_config.json` should look like:

```json
{
  "mcpServers": {
    "hostinger-mcp": {
      "type": "stdio",
      "command": "npx",
      "args": ["hostinger-api-mcp@latest"],
      "env": {
        "API_TOKEN": "${HOSTINGER_API_KEY}"
      }
    },
    "hermes-rolodex": {
      "type": "stdio",
      "command": "python3",
      "args": ["/opt/pauli-hermes-agent/mcp-servers/hermes-rolodex/server.py"],
      "env": {
        "HERMES_CONFIG": "~/.hermes/config.yaml"
      }
    },
    "filesystem": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/"]
    }
  }
}
```

---

## Ask Claude to Deploy

Once MCP is connected, you can ask Claude directly:

```
"Use Hostinger MCP and the deploy-hostinger.sh script to:
1. SSH into my VPS at 31.220.58.212
2. Run the deployment script
3. Configure all API keys
4. Start the services
5. Verify everything is working
6. Give me the status"
```

Claude will use the MCP tools to execute everything automatically.

---

## Troubleshooting

**"hostinger-mcp not found"**
- Make sure you have `npx` installed (comes with Node.js)
- Run: `npm install -g @modelcontextprotocol/server-nodejs`

**"API connection failed"**
- Verify your API token: `${HOSTINGER_API_KEY}`
- Check that it matches your Hostinger account

**MCP not loading in Claude**
- Restart Claude Desktop completely
- Check the config file syntax (use a JSON validator)
- View Claude logs for error messages

---

## Next Steps

1. **Add the config** to your Claude Desktop config file
2. **Restart Claude** Desktop
3. **Tell Claude:** "Deploy my Hermes Agent using Hostinger MCP"
4. **Watch it happen** automatically!

---

That's it. You now have direct VPS control from Claude. 🚀
