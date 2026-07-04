// agent.ts — Vercel AI SDK + Composio
// See Composio docs: https://docs.composio.dev/

import { anthropic } from "@ai-sdk/anthropic";
import { Composio } from "@composio/core";
import { VercelProvider } from "@composio/vercel";
import { stepCountIs, streamText } from "ai";

// Initialize Composio with Vercel provider
// Get your API key from: https://dashboard.composio.dev/executiveusa/HERMES/settings/api-keys
const composio = new Composio({ 
  provider: new VercelProvider(),
  apiKey: process.env.COMPOSIO_API_KEY 
});

const userId = "hermes_user";

async function main() {
  // Create a tool router session
  const session = await composio.create(userId);
  const tools = await session.tools();

  // Stream text with the agent
  const stream = await streamText({
    model: anthropic("claude-sonnet-4-20250514"),
    prompt: "Star the composiohq/composio repo on GitHub",
    stopWhen: stepCountIs(10),
    tools,
  });

  // Stream the output
  for await (const textPart of stream.textStream) {
    process.stdout.write(textPart);
  }
}

main().catch(console.error);
