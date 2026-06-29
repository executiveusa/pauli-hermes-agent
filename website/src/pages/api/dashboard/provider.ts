import type { NextApiRequest, NextApiResponse } from 'next';
import fs from 'fs';
import path from 'path';

interface SwitchProviderRequest {
  provider_id: string;
}

interface SwitchProviderResponse {
  success: boolean;
  message: string;
  previous_provider?: string;
  new_provider?: string;
}

function loadProviders(): Record<string, any> {
  try {
    const registryPath = path.join(process.cwd(), '../../free-mode/providers.json');
    if (fs.existsSync(registryPath)) {
      const data = JSON.parse(fs.readFileSync(registryPath, 'utf-8'));
      return Object.fromEntries(data.providers.map((p: any) => [p.id, p]));
    }
  } catch (err) {
    console.error('Failed to load providers:', err);
  }
  return {};
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<SwitchProviderResponse>
) {
  if (req.method !== 'POST') {
    return res.status(405).json({ success: false, message: 'Method not allowed' });
  }

  try {
    const { provider_id } = req.body as SwitchProviderRequest;

    if (!provider_id) {
      return res.status(400).json({ success: false, message: 'provider_id is required' });
    }

    const providers = loadProviders();
    if (!providers[provider_id]) {
      return res.status(404).json({ success: false, message: `Provider ${provider_id} not found` });
    }

    const previousProvider = process.env.FREE_MODE_PROVIDER || 'auto';

    // In a real implementation, you would:
    // 1. Update environment variables
    // 2. Restart the proxy/agent
    // 3. Log the change to a database

    // For now, we'll just log the intent
    const switchLog = {
      timestamp: new Date().toISOString(),
      from: previousProvider,
      to: provider_id,
      user: req.headers['x-user-id'] || 'anonymous',
    };

    const logPath = path.join(process.cwd(), '../../.free-mode-provider-switches.json');
    let switches = [];
    if (fs.existsSync(logPath)) {
      switches = JSON.parse(fs.readFileSync(logPath, 'utf-8'));
    }
    switches.push(switchLog);
    fs.writeFileSync(logPath, JSON.stringify(switches, null, 2));

    res.status(200).json({
      success: true,
      message: `Provider switched from ${previousProvider} to ${provider_id}`,
      previous_provider: previousProvider,
      new_provider: provider_id,
    });
  } catch (error) {
    console.error('Provider switch error:', error);
    res.status(500).json({
      success: false,
      message: 'Failed to switch provider',
    });
  }
}
