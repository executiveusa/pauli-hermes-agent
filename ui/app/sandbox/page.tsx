'use client';

import { useEffect, useState } from 'react';
import { SandcastleProvider, SandcastleRun, SandcastleHealth } from '@/lib/schemas/sandcastle';
import SandcastleRunDetail from '@/components/SandcastleRunDetail';

export default function SandboxPage() {
  const [health, setHealth] = useState<SandcastleHealth | null>(null);
  const [providers, setProviders] = useState<SandcastleProvider[]>([]);
  const [runs, setRuns] = useState<SandcastleRun[]>([]);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [statusRes, runsRes] = await Promise.all([
          fetch('/api/sandcastle/status'),
          fetch('/api/sandcastle/runs?status=running'),
        ]);

        if (!statusRes.ok) throw new Error('Failed to fetch status');
        const statusData = await statusRes.json();
        setProviders(statusData.providers);
        setHealth(statusData);

        if (runsRes.ok) {
          const runsData = await runsRes.json();
          setRuns(runsData.runs || []);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 3000);
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

      {/* Active Runs */}
      {runs.length > 0 && (
        <div style={{ marginBottom: '28px' }}>
          <div style={{ fontSize: '12px', fontWeight: 600, color: '#444', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '12px' }}>
            Active Runs
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))', gap: '12px' }}>
            {runs.map(run => (
              <div
                key={run.id}
                onClick={() => setSelectedRunId(run.id)}
                style={{
                  background: '#16161f',
                  border: '1px solid #252535',
                  borderRadius: '8px',
                  padding: '16px',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = '#3a5f99';
                  e.currentTarget.style.background = '#1a1a28';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = '#252535';
                  e.currentTarget.style.background = '#16161f';
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
                  <h3 style={{ margin: 0, fontSize: '14px', fontWeight: 600, color: '#eee' }}>{run.title}</h3>
                  <span
                    style={{
                      padding: '2px 8px',
                      borderRadius: '4px',
                      background:
                        run.status === 'completed'
                          ? '#22c55e22'
                          : run.status === 'failed' || run.status === 'discarded'
                          ? '#ef444422'
                          : '#3a5f9922',
                      color:
                        run.status === 'completed'
                          ? '#22c55e'
                          : run.status === 'failed' || run.status === 'discarded'
                          ? '#ef4444'
                          : '#7aa0cc',
                      fontSize: '11px',
                      fontWeight: 600,
                    }}
                  >
                    {run.status.replace(/_/g, ' ').toUpperCase()}
                  </span>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '8px', fontSize: '11px' }}>
                  <div>
                    <div style={{ color: '#555', marginBottom: '2px' }}>Branch</div>
                    <div style={{ color: '#ccc', fontFamily: 'monospace', fontSize: '10px' }}>{run.branch}</div>
                  </div>
                  <div>
                    <div style={{ color: '#555', marginBottom: '2px' }}>Provider</div>
                    <div style={{ color: '#ccc', fontSize: '10px' }}>{run.provider}</div>
                  </div>
                  <div>
                    <div style={{ color: '#555', marginBottom: '2px' }}>Files</div>
                    <div style={{ color: '#ccc' }}>{run.changedFiles.length}</div>
                  </div>
                  <div>
                    <div style={{ color: '#555', marginBottom: '2px' }}>Commits</div>
                    <div style={{ color: '#ccc' }}>{run.commits.length}</div>
                  </div>
                </div>

                <div style={{ marginTop: '12px', fontSize: '11px', color: '#555' }}>
                  Click to view details
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

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

      {/* Run Detail Panel */}
      {selectedRunId && (
        <SandcastleRunDetail
          runId={selectedRunId}
          onClose={() => setSelectedRunId(null)}
        />
      )}
    </div>
  );
}
