'use client';

import { useEffect, useState } from 'react';
import { SandcastleProvider, SandcastleRun, SandcastleHealth } from '@/lib/schemas/sandcastle';

export default function SandboxPage() {
  const [health, setHealth] = useState<SandcastleHealth | null>(null);
  const [providers, setProviders] = useState<SandcastleProvider[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const res = await fetch('/api/sandcastle/status');
        if (!res.ok) throw new Error('Failed to fetch status');
        const data = await res.json();
        setProviders(data.providers);
        setHealth(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    fetchStatus();
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div style={{ padding: '32px', textAlign: 'center' }}>
        <div style={{ fontSize: '18px', fontWeight: 600, color: '#ccc' }}>Loading Sandbox...</div>
      </div>
    );
  }

  return (
    <div style={{ padding: '24px 16px', maxWidth: '1200px', margin: '0 auto', width: '100%' }}>
      {/* Hero */}
      <div style={{ marginBottom: '32px' }}>
        <h1 style={{ fontSize: '28px', fontWeight: 700, margin: 0, color: '#eee' }}>
          Safe Agent Runs
        </h1>
        <p style={{ margin: '8px 0 0', color: '#555', fontSize: '14px' }}>
          Let coding agents work in isolated branches while you watch every step.
        </p>
      </div>

      {/* Error Banner */}
      {error && (
        <div style={{
          background: 'rgba(220,38,38,0.1)',
          border: '1px solid rgba(220,38,38,0.3)',
          borderRadius: '8px',
          padding: '16px',
          marginBottom: '24px',
          color: '#ff6b6b',
          fontSize: '13px',
        }}>
          {error}
        </div>
      )}

      {/* Provider Status Grid */}
      <div style={{ marginBottom: '28px' }}>
        <div style={{ fontSize: '12px', fontWeight: 600, color: '#444', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '12px' }}>
          Provider Status
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '12px' }}>
          {providers.map(provider => (
            <div
              key={provider.id}
              style={{
                background: '#16161f',
                border: `1px solid ${provider.healthy ? '#2a3f2a' : '#3f2a2a'}`,
                borderRadius: '8px',
                padding: '16px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                <span style={{ fontSize: '14px', fontWeight: 600, color: provider.healthy ? '#4ade80' : '#ef5350' }}>
                  {provider.healthy ? '🟢' : '🔴'} {provider.label}
                </span>
              </div>
              <div style={{ fontSize: '12px', color: '#555', marginBottom: '8px' }}>
                {provider.description}
              </div>
              {!provider.healthy && provider.missingRequirements && (
                <div style={{ fontSize: '11px', color: '#ff6b6b', marginTop: '8px' }}>
                  Missing: {provider.missingRequirements.join(', ')}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Stats */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '12px', marginBottom: '28px' }}>
        {[
          { label: 'Healthy Providers', value: providers.filter(p => p.healthy).length },
          { label: 'Active Runs', value: health?.activeRuns || 0 },
          { label: 'Total Runs', value: health?.totalRuns || 0 },
        ].map(({ label, value }) => (
          <div key={label} style={{ background: '#16161f', border: '1px solid #252535', borderRadius: '8px', padding: '14px', textAlign: 'center' }}>
            <div style={{ fontSize: '20px', fontWeight: 700, color: '#ccc' }}>{value}</div>
            <div style={{ fontSize: '11px', color: '#555', marginTop: '4px' }}>{label}</div>
          </div>
        ))}
      </div>

      {/* Explainer */}
      <div style={{ background: '#16161f', border: '1px solid #252535', borderRadius: '8px', padding: '16px' }}>
        <div style={{ fontSize: '12px', fontWeight: 600, color: '#444', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '12px' }}>
          How It Works
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '12px' }}>
          {[
            { number: '1', title: 'Create Task', desc: 'Hermes creates a bead' },
            { number: '2', title: 'Branch', desc: 'Sandcastle creates isolated branch' },
            { number: '3', title: 'Execute', desc: 'Agent works safely' },
            { number: '4', title: 'Test', desc: 'Tests run automatically' },
            { number: '5', title: 'Review', desc: 'You review commits' },
            { number: '6', title: 'Approve', desc: 'Approve merge/deploy' },
          ].map(({ number, title, desc }) => (
            <div key={number} style={{ fontSize: '12px' }}>
              <div style={{ fontSize: '16px', fontWeight: 700, color: '#3a5f99', marginBottom: '4px' }}>{number}</div>
              <div style={{ fontSize: '12px', fontWeight: 600, color: '#ccc' }}>{title}</div>
              <div style={{ fontSize: '11px', color: '#555', marginTop: '2px' }}>{desc}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
