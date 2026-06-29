import { useEffect, useState } from 'react';
import Head from 'next/head';

interface ProviderStatus {
  id: string;
  label: string;
  status: 'healthy' | 'missing_secret' | 'auth_failed' | 'timeout' | 'failed' | 'unknown';
  latency_ms?: number;
  connected: boolean;
  model?: string;
}

interface CostData {
  provider: string;
  requests: number;
  cost: number;
  timestamp: string;
}

interface DashboardData {
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

export default function Dashboard() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
  const [costHistory, setCostHistory] = useState<CostData[]>([]);

  useEffect(() => {
    fetchDashboardData();
    const interval = setInterval(fetchDashboardData, 5000); // Refresh every 5s
    return () => clearInterval(interval);
  }, []);

  const fetchDashboardData = async () => {
    try {
      const res = await fetch('/api/dashboard/status');
      if (!res.ok) throw new Error('Failed to fetch dashboard data');
      const dashboardData = await res.json();
      setData(dashboardData);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const fetchCostHistory = async () => {
    try {
      const res = await fetch('/api/dashboard/costs');
      if (!res.ok) throw new Error('Failed to fetch costs');
      const costs = await res.json();
      setCostHistory(costs);
    } catch (err) {
      console.error('Failed to fetch costs:', err);
    }
  };

  const switchProvider = async (providerId: string) => {
    try {
      const res = await fetch('/api/dashboard/provider', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ provider_id: providerId }),
      });
      if (!res.ok) throw new Error('Failed to switch provider');
      setSelectedProvider(providerId);
      fetchDashboardData();
    } catch (err) {
      console.error('Failed to switch provider:', err);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-slate-950">
        <div className="text-cyan-400">Loading dashboard...</div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-slate-950">
        <div className="text-red-400">Error: {error || 'No data'}</div>
      </div>
    );
  }

  return (
    <>
      <Head>
        <title>Hermes FREE MODE Dashboard</title>
        <meta name="description" content="FREE MODE control panel and monitoring" />
      </Head>

      <div className="min-h-screen bg-slate-950 text-slate-100 p-8">
        <div className="max-w-7xl mx-auto">
          {/* Header */}
          <div className="mb-8">
            <h1 className="text-4xl font-bold text-cyan-400 mb-2">FREE MODE Dashboard</h1>
            <p className="text-slate-400">Universal LiteLLM proxy gateway for free/local AI inference</p>
          </div>

          {/* Status Cards */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
            {/* FREE MODE Status */}
            <div className="bg-slate-900 border border-cyan-500 rounded-lg p-6">
              <div className="text-sm text-slate-400 mb-2">FREE MODE</div>
              <div className="text-2xl font-bold">
                <span className={data.free_mode_enabled ? 'text-green-400' : 'text-red-400'}>
                  {data.free_mode_enabled ? '✓ ENABLED' : '✗ DISABLED'}
                </span>
              </div>
            </div>

            {/* Proxy Status */}
            <div className="bg-slate-900 border border-cyan-500 rounded-lg p-6">
              <div className="text-sm text-slate-400 mb-2">Proxy Status</div>
              <div className="text-2xl font-bold">
                <span className={data.proxy_reachable ? 'text-green-400' : 'text-red-400'}>
                  {data.proxy_reachable ? '✓ Online' : '✗ Offline'}
                </span>
              </div>
              {data.proxy_latency_ms && (
                <div className="text-xs text-slate-400 mt-2">{data.proxy_latency_ms}ms latency</div>
              )}
            </div>

            {/* Current Provider */}
            <div className="bg-slate-900 border border-cyan-500 rounded-lg p-6">
              <div className="text-sm text-slate-400 mb-2">Current Provider</div>
              <div className="text-xl font-bold text-cyan-400">{data.current_provider.toUpperCase()}</div>
              <div className="text-xs text-slate-400 mt-2">{data.current_model}</div>
            </div>

            {/* Cost Today */}
            <div className="bg-slate-900 border border-cyan-500 rounded-lg p-6">
              <div className="text-sm text-slate-400 mb-2">Cost Today</div>
              <div className="text-2xl font-bold text-green-400">${data.costs_today.toFixed(2)}</div>
              <div className="text-xs text-slate-400 mt-2">Month: ${data.costs_month.toFixed(2)}</div>
            </div>
          </div>

          {/* Provider Switcher */}
          <div className="bg-slate-900 border border-slate-700 rounded-lg p-6 mb-8">
            <h2 className="text-xl font-bold text-cyan-400 mb-4">Available Providers</h2>
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
              {data.providers
                .filter((p) => p.connected)
                .map((provider) => (
                  <button
                    key={provider.id}
                    onClick={() => switchProvider(provider.id)}
                    className={`p-3 rounded border transition-colors ${
                      data.current_provider === provider.id
                        ? 'bg-cyan-500 border-cyan-400 text-slate-950 font-bold'
                        : 'bg-slate-800 border-slate-600 hover:border-cyan-400 hover:text-cyan-400'
                    }`}
                  >
                    <div className="font-semibold">{provider.label}</div>
                    <div className="text-xs text-slate-400 mt-1">
                      {provider.latency_ms ? `${provider.latency_ms}ms` : 'checking...'}
                    </div>
                  </button>
                ))}
            </div>
          </div>

          {/* Provider Health */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8">
            {/* All Providers */}
            <div className="bg-slate-900 border border-slate-700 rounded-lg p-6">
              <h2 className="text-xl font-bold text-cyan-400 mb-4">Provider Health</h2>
              <div className="space-y-2 max-h-96 overflow-y-auto">
                {data.providers.map((provider) => (
                  <div
                    key={provider.id}
                    className="flex items-center justify-between p-2 bg-slate-800 rounded"
                  >
                    <div>
                      <div className="font-semibold">{provider.label}</div>
                      <div className="text-xs text-slate-400">{provider.id}</div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span
                        className={`px-2 py-1 rounded text-xs font-semibold ${
                          provider.status === 'healthy'
                            ? 'bg-green-500/20 text-green-400'
                            : provider.status === 'missing_secret'
                              ? 'bg-yellow-500/20 text-yellow-400'
                              : 'bg-red-500/20 text-red-400'
                        }`}
                      >
                        {provider.status}
                      </span>
                      {provider.latency_ms && (
                        <span className="text-xs text-slate-400">{provider.latency_ms}ms</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Statistics */}
            <div className="bg-slate-900 border border-slate-700 rounded-lg p-6">
              <h2 className="text-xl font-bold text-cyan-400 mb-4">Statistics</h2>
              <div className="space-y-4">
                <div>
                  <div className="text-sm text-slate-400 mb-1">Healthy Providers</div>
                  <div className="text-2xl font-bold text-green-400">
                    {data.providers.filter((p) => p.status === 'healthy').length}/{data.providers.length}
                  </div>
                </div>
                <div>
                  <div className="text-sm text-slate-400 mb-1">Requests Today</div>
                  <div className="text-2xl font-bold text-cyan-400">{data.recent_requests}</div>
                </div>
                <div>
                  <div className="text-sm text-slate-400 mb-1">Average Latency</div>
                  <div className="text-2xl font-bold text-blue-400">
                    {data.proxy_latency_ms ? `${data.proxy_latency_ms}ms` : 'N/A'}
                  </div>
                </div>
                <button
                  onClick={fetchCostHistory}
                  className="w-full mt-4 bg-cyan-500 hover:bg-cyan-600 text-slate-950 font-bold py-2 px-4 rounded transition-colors"
                >
                  Load Cost History
                </button>
              </div>
            </div>
          </div>

          {/* Cost History */}
          {costHistory.length > 0 && (
            <div className="bg-slate-900 border border-slate-700 rounded-lg p-6">
              <h2 className="text-xl font-bold text-cyan-400 mb-4">Cost History (Last 24h)</h2>
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-slate-700">
                      <th className="text-left py-2 px-4 text-slate-400">Provider</th>
                      <th className="text-right py-2 px-4 text-slate-400">Requests</th>
                      <th className="text-right py-2 px-4 text-slate-400">Cost</th>
                      <th className="text-left py-2 px-4 text-slate-400">Time</th>
                    </tr>
                  </thead>
                  <tbody>
                    {costHistory.map((entry, idx) => (
                      <tr key={idx} className="border-b border-slate-800 hover:bg-slate-800">
                        <td className="py-2 px-4 text-cyan-400">{entry.provider}</td>
                        <td className="text-right py-2 px-4">{entry.requests}</td>
                        <td className="text-right py-2 px-4 text-green-400">${entry.cost.toFixed(4)}</td>
                        <td className="py-2 px-4 text-slate-400">
                          {new Date(entry.timestamp).toLocaleString()}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      </div>
    </>
  );
}
