# Chat Integrations

## Built-in Chat Interfaces

### Web Chat

The web chat interface is included out of the box at your APP_URL. No additional configuration needed.

- **Streaming responses** — AI responses stream in real-time via the Vercel AI SDK
- **File uploads** — Send images, PDFs, and text files for the AI to process
- **Chat history** — Browse past conversations grouped by date, resume any chat
- **Job management** — Create and monitor agent jobs from the Swarm page
- **Notifications** — Job completion alerts with unread badges
- **API key management** — Generate and manage API keys from Settings

### Telegram (Optional)

Connect a Telegram bot to chat with your agent on the go:

```bash
npm run setup-telegram
```

The setup wizard configures your bot token, webhook, and chat ID. Once connected, message your bot directly to chat or create jobs. Supports text, voice messages (transcribed via OpenAI Whisper), photos, and documents.

See [Configuration](CONFIGURATION.md) for manual Telegram setup instructions.

---

## Channel Adapter Architecture

thepopebot uses a channel adapter pattern to normalize messages across different chat platforms. The AI layer is channel-agnostic — it receives the same normalized message format regardless of the source.

### Base Class

`lib/channels/base.js` defines the `ChannelAdapter` interface:

| Method | Description |
|--------|-------------|
| `receive(request)` | Parse incoming webhook into normalized message data (or `null` to ignore) |
| `acknowledge(metadata)` | Show message receipt (e.g., Telegram thumbs-up reaction) |
| `startProcessingIndicator(metadata)` | Show activity while AI processes (e.g., typing indicator). Returns a stop function |
| `sendResponse(threadId, text, metadata)` | Send a complete response back to the channel |
| `supportsStreaming` (getter) | Whether the channel supports real-time streaming (e.g., web chat) |

### Normalized Message Format

All adapters return the same shape from `receive()`:

```javascript
{
  threadId: string,      // Channel-specific thread/chat identifier
  text: string,          // Message text (voice messages are pre-transcribed)
  attachments: [         // Non-text content for the AI
    { category: "image", mimeType: "image/jpeg", data: Buffer },
    { category: "document", mimeType: "application/pdf", data: Buffer }
  ],
  metadata: object       // Channel-specific data (message IDs, chat IDs, etc.)
}
```

Voice/audio messages are fully resolved by the adapter — transcribed to text and included in the `text` field, not passed as attachments.

### Reference Implementation

`lib/channels/telegram.js` (`TelegramAdapter`) is the reference implementation. It handles:
- Webhook secret validation
- Chat ID authorization
- Text, voice/audio (Whisper transcription), photo, and document messages
- Thumbs-up reaction on receipt, typing indicator during processing

---

## Adding a New Channel

To add a new chat channel (e.g., Discord, Slack, WhatsApp):

1. **Create an adapter** extending `ChannelAdapter` in `lib/channels/`:

```javascript
import { ChannelAdapter } from './base.js';

class DiscordAdapter extends ChannelAdapter {
  async receive(request) {
    // Parse the incoming webhook, validate auth, return normalized message
    // Return null to ignore the message
  }

  async acknowledge(metadata) {
    // Optional: react to the message
  }

  startProcessingIndicator(metadata) {
    // Optional: show typing indicator
    return () => {}; // Return stop function
  }

  async sendResponse(threadId, text, metadata) {
    // Send the AI's response back to the channel
  }
}
```

2. **Add a factory function** in `lib/channels/index.js`:

```javascript
import { DiscordAdapter } from './discord.js';

export function getDiscordAdapter(botToken) {
  // Lazy singleton pattern (see getTelegramAdapter for reference)
}
```

3. **Add a webhook route** in `api/index.js` to handle incoming messages from the new channel.

4. **The AI layer needs zero changes** — it's channel-agnostic. It receives normalized messages and returns responses regardless of the source channel.

---

## Potential Integrations

The adapter pattern makes it straightforward to add any channel that supports webhooks:

- **Discord** — Bot webhooks, slash commands
- **Slack** — Events API, slash commands
- **WhatsApp** — Business API webhooks
- **SMS** — Twilio webhooks
- **Email** — Inbound email parsing (SendGrid, Mailgun)

All follow the same pattern: receive webhook, normalize to `{ threadId, text, attachments, metadata }`, send response back.
