import type { NextApiRequest, NextApiResponse } from 'next';
import fs from 'fs';
import path from 'path';

interface DashboardStatus {
  free_mode_enabled: boolean;
  current_provider: string;
  current_model: string;
  proxy_reachable: boolean;
  proxy_latency_ms?: number;
  providers: ProviderStatus[];
  costs_today: number;
  costs_month: number;
  recent_requests: number;
}

interface ProviderStatus {
  id: string;
  label: string;
  status: 'healthy' | 'missing_secret' | 'auth_failed' | 'timeout' | 'failed' | 'unknown';
  latency_ms?: number;
  connected: boolean;
  model?: string;
}

// Load providers from registry
function loadProviders(): ProviderStatus[] {
  try {
    const registryPath = path.join(process.cwd(), '../../free-mode/providers.json');
    if (fs.existsSync(registryPath)) {
      const data = JSON.parse(fs.readFileSync(registryPath, 'utf-8'));
      return data.providers.map((p: any) => ({
        id: p.id,
        label: p.label,
        status: 'unknown' as const,
        connected: false,
        model: p.model_env,
      }));
    }
  } catch (err) {
    console.error('Failed to load providers:', err);
  }
  return [];
}

// Check proxy health
async function checkProxyHealth(): Promise<{ reachable: boolean; latency?: number }> {
  try {
    const start = Date.now();
    const res = await fetch('http://127.0.0.1:4000/health', {
      timeout: 5000,
    });
    const latency = Date.now() - start;
    return {
      reachable: res.ok,
      latency,
    };
  } catch (err) {
    return { reachable: false };
  }
}

// Get costs from file-based store
function getCosts(): { today: number; month: number; requests: number } {
  try {
    const costsPath = path.join(process.cwd(), '../../.free-mode-costs.json');
    if (fs.existsSync(costsPath)) {
      const data = JSON.parse(fs.readFileSync(costsPath, 'utf-8'));
      const now = new Date();
      const today = data.daily[now.toISOString().split('T')[0]] || 0;
      const monthKey = now.toISOString().slice(0, 7);
      const month = data.monthly[monthKey] || 0;
      const requests = data.requests || 0;
      return { today, month, requests };
    }
  } catch (err) {
    console.error('Failed to load costs:', err);
  }
  return { today: 0, month: 0, requests: 0 };
}

export default async function handler(req: NextApiRequest, res: NextApiResponse<DashboardStatus | { error: string }>) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const freeMode = process.env.FREE_MODE === 'true';
    const currentProvider = process.env.FREE_MODE_PROVIDER || 'auto';
    const currentModel = process.env.FREE_MODE_MODEL || 'free-auto';

    const proxyHealth = await checkProxyHealth();
    const providers = loadProviders();
    const costs = getCosts();

    // Update provider statuses based on health check
    const healthyProviders = providers.map((p) => ({
      ...p,
      connected: proxyHealth.reachable,
      status: proxyHealth.reachable ? 'healthy' : ('failed' as const),
      latency_ms: proxyHealth.latency,
    }));

    const status: DashboardStatus = {
      free_mode_enabled: freeMode,
      current_provider: currentProvider,
      current_model: currentModel,
      proxy_reachable: proxyHealth.reachable,
      proxy_latency_ms: proxyHealth.latency,
      providers: healthyProviders,
      costs_today: costs.today,
      costs_month: costs.month,
      recent_requests: costs.requests,
    };

    res.status(200).json(status);
  } catch (error) {
    console.error('Dashboard status error:', error);
    res.status(500).json({ error: 'Failed to fetch dashboard status' });
  }
}
