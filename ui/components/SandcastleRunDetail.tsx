'use client';

import { useState, useEffect } from 'react';
import { SandcastleRun, SandcastleEvent } from '@/lib/schemas/sandcastle';
import SandcastleEventTimeline from './SandcastleEventTimeline';
import SandcastleLogStream from './SandcastleLogStream';

interface SandcastleRunDetailProps {
  runId: string;
  onClose: () => void;
}

type Tab = 'overview' | 'timeline' | 'logs' | 'commits' | 'files' | 'tests';

export default function SandcastleRunDetail({ runId, onClose }: SandcastleRunDetailProps) {
  const [run, setRun] = useState<SandcastleRun | null>(null);
  const [events, setEvents] = useState<SandcastleEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<Tab>('overview');
  const [pollInterval, setPollInterval] = useState(2000);

  useEffect(() => {
    const fetchRun = async () => {
      try {
        const [runRes, eventsRes] = await Promise.all([
          fetch(`/api/sandcastle/runs/${runId}`),
          fetch(`/api/sandcastle/runs/${runId}/events`),
        ]);

        if (runRes.ok && eventsRes.ok) {
          const runData = await runRes.json();
          const eventsData = await eventsRes.json();
          setRun(runData);
          setEvents(eventsData.events || []);

          // Stop polling if run is completed
          if (
            runData.status === 'completed' ||
            runData.status === 'failed' ||
            runData.status === 'stopped' ||
            runData.status === 'discarded'
          ) {
            setPollInterval(0);
          }
        }
      } catch (err) {
        console.error('Failed to fetch run details:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchRun();

    if (pollInterval > 0) {
      const timer = setInterval(fetchRun, pollInterval);
      return () => clearInterval(timer);
    }
  }, [runId, pollInterval]);

  const handleAction = async (action: 'stop' | 'retry' | 'approve' | 'discard') => {
    try {
      const endpoints: Record<string, string> = {
        stop: `/api/sandcastle/runs/${runId}/stop`,
        retry: `/api/sandcastle/runs/${runId}/retry`,
        approve: `/api/sandcastle/runs/${runId}/approve-merge`,
        discard: `/api/sandcastle/runs/${runId}/discard`,
      };

      const res = await fetch(endpoints[action], {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
      });

      if (res.ok) {
        // Refetch run details
        const runRes = await fetch(`/api/sandcastle/runs/${runId}`);
        if (runRes.ok) {
          setRun(await runRes.json());
        }
      }
    } catch (err) {
      console.error(`Failed to ${action} run:`, err);
    }
  };

  const statusColor =
    run?.status === 'completed'
      ? '#22c55e'
      : run?.status === 'failed' || run?.status === 'discarded'
      ? '#ef4444'
      : run?.status === 'running' || run?.status === 'tests_running'
      ? '#3a5f99'
      : run?.status === 'stopped'
      ? '#ef4444'
      : '#eab308';

  if (loading) {
    return (
      <div
        style={{
          position: 'fixed',
          right: 0,
          top: 0,
          width: '450px',
          height: '100vh',
          background: '#16161f',
          borderLeft: '1px solid #252535',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#555',
        }}
      >
        Loading...
      </div>
    );
  }

  if (!run) {
    return (
      <div
        style={{
          position: 'fixed',
          right: 0,
          top: 0,
          width: '450px',
          height: '100vh',
          background: '#16161f',
          borderLeft: '1px solid #252535',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#ef4444',
        }}
      >
        Run not found
      </div>
    );
  }

  const tabs: { id: Tab; label: string }[] = [
    { id: 'overview', label: 'Overview' },
    { id: 'timeline', label: 'Timeline' },
    { id: 'logs', label: 'Logs' },
    { id: 'commits', label: 'Commits' },
    { id: 'files', label: 'Files' },
  ];

  return (
    <div
      style={{
        position: 'fixed',
        right: 0,
        top: 0,
        width: '450px',
        height: '100vh',
        background: '#16161f',
        borderLeft: '1px solid #252535',
        display: 'flex',
        flexDirection: 'column',
        zIndex: 1000,
      }}
    >
      {/* Header */}
      <div style={{ padding: '16px', borderBottom: '1px solid #252535' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
          <h2 style={{ margin: 0, fontSize: '16px', fontWeight: 600, color: '#eee' }}>Run Details</h2>
          <button
            onClick={onClose}
            style={{
              fontSize: '20px',
              background: 'none',
              border: 'none',
              color: '#555',
              cursor: 'pointer',
            }}
            title="Close"
          >
            ✕
          </button>
        </div>

        <div style={{ fontSize: '12px', color: '#777', marginBottom: '8px' }}>
          {run.id.slice(0, 12)}...
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
          <span
            style={{
              padding: '2px 8px',
              borderRadius: '4px',
              background: statusColor + '22',
              color: statusColor,
              fontSize: '11px',
              fontWeight: 600,
            }}
          >
            {run.status.toUpperCase()}
          </span>
          <span style={{ fontSize: '12px', color: '#777' }}>{run.title}</span>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '8px' }}>
          {[
            { label: 'Branch', value: run.branch },
            { label: 'Provider', value: run.provider },
            { label: 'Files', value: run.changedFiles.length },
            { label: 'Commits', value: run.commits.length },
          ].map(({ label, value }) => (
            <div key={label} style={{ fontSize: '11px' }}>
              <div style={{ color: '#555', marginBottom: '2px' }}>{label}</div>
              <div style={{ color: '#ccc', wordBreak: 'break-word' }}>{value}</div>
            </div>
          ))}
        </div>
      </div>

      {/* Tabs */}
      <div
        style={{
          display: 'flex',
          gap: '8px',
          padding: '8px 16px',
          borderBottom: '1px solid #252535',
          overflow: 'auto',
        }}
      >
        {tabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            style={{
              padding: '6px 12px',
              fontSize: '12px',
              fontWeight: activeTab === tab.id ? 600 : 400,
              border: `1px solid ${activeTab === tab.id ? '#3a5f99' : '#252535'}`,
              borderRadius: '4px',
              background: activeTab === tab.id ? '#3a5f9944' : 'transparent',
              color: activeTab === tab.id ? '#7aa0cc' : '#555',
              cursor: 'pointer',
              whiteSpace: 'nowrap',
            }}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: 'auto', padding: '16px' }}>
        {activeTab === 'overview' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <div>
              <div style={{ fontSize: '11px', color: '#555', marginBottom: '4px' }}>PROMPT</div>
              <div
                style={{
                  padding: '8px 10px',
                  background: '#0d0d14',
                  borderRadius: '6px',
                  fontSize: '11px',
                  color: '#888',
                  fontFamily: 'monospace',
                  maxHeight: '120px',
                  overflow: 'auto',
                  whiteSpace: 'pre-wrap',
                  wordBreak: 'break-word',
                }}
              >
                {run.prompt}
              </div>
            </div>

            {run.approvalStatus && (
              <div>
                <div style={{ fontSize: '11px', color: '#555', marginBottom: '8px' }}>APPROVAL STATUS</div>
                <span
                  style={{
                    padding: '4px 8px',
                    borderRadius: '4px',
                    background:
                      run.approvalStatus === 'approved'
                        ? '#22c55e22'
                        : run.approvalStatus === 'rejected'
                        ? '#ef444422'
                        : '#eab30822',
                    color:
                      run.approvalStatus === 'approved'
                        ? '#22c55e'
                        : run.approvalStatus === 'rejected'
                        ? '#ef4444'
                        : '#eab308',
                    fontSize: '11px',
                    fontWeight: 600,
                  }}
                >
                  {run.approvalStatus.toUpperCase()}
                </span>
              </div>
            )}
          </div>
        )}

        {activeTab === 'timeline' && <SandcastleEventTimeline events={events} loading={false} />}

        {activeTab === 'logs' && <SandcastleLogStream events={events} />}

        {activeTab === 'commits' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {run.commits.length === 0 ? (
              <div style={{ color: '#555', textAlign: 'center', padding: '20px' }}>No commits</div>
            ) : (
              run.commits.map(commit => (
                <div
                  key={commit.sha}
                  style={{
                    padding: '10px',
                    background: '#0d0d14',
                    borderRadius: '6px',
                    fontSize: '11px',
                  }}
                >
                  <div style={{ color: '#ccc', fontFamily: 'monospace', marginBottom: '4px' }}>
                    {commit.sha.slice(0, 8)}
                  </div>
                  <div style={{ color: '#7aa0cc', marginBottom: '4px' }}>{commit.message}</div>
                  <div style={{ color: '#555', fontSize: '10px' }}>
                    {commit.author} • {new Date(commit.timestamp).toLocaleString()}
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'files' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {run.changedFiles.length === 0 ? (
              <div style={{ color: '#555', textAlign: 'center', padding: '20px' }}>No files changed</div>
            ) : (
              run.changedFiles.map(file => (
                <div
                  key={file}
                  style={{
                    padding: '10px',
                    background: '#0d0d14',
                    borderRadius: '6px',
                    fontSize: '11px',
                    color: '#888',
                    fontFamily: 'monospace',
                  }}
                >
                  {file}
                </div>
              ))
            )}
          </div>
        )}
      </div>

      {/* Action buttons */}
      <div style={{ padding: '16px', borderTop: '1px solid #252535', display: 'flex', gap: '8px' }}>
        {run.status === 'running' && (
          <button
            onClick={() => handleAction('stop')}
            style={{
              flex: 1,
              padding: '8px 12px',
              fontSize: '12px',
              fontWeight: 600,
              border: '1px solid #ef4444',
              borderRadius: '6px',
              background: '#ef444422',
              color: '#ef4444',
              cursor: 'pointer',
            }}
          >
            Stop Run
          </button>
        )}

        {run.status === 'failed' && (
          <>
            <button
              onClick={() => handleAction('retry')}
              style={{
                flex: 1,
                padding: '8px 12px',
                fontSize: '12px',
                fontWeight: 600,
                border: '1px solid #3a5f99',
                borderRadius: '6px',
                background: '#3a5f9944',
                color: '#7aa0cc',
                cursor: 'pointer',
              }}
            >
              Retry
            </button>
            <button
              onClick={() => handleAction('discard')}
              style={{
                flex: 1,
                padding: '8px 12px',
                fontSize: '12px',
                fontWeight: 600,
                border: '1px solid #ef4444',
                borderRadius: '6px',
                background: '#ef444422',
                color: '#ef4444',
                cursor: 'pointer',
              }}
            >
              Discard
            </button>
          </>
        )}

        {run.status === 'needs_approval' && (
          <>
            <button
              onClick={() => handleAction('approve')}
              style={{
                flex: 1,
                padding: '8px 12px',
                fontSize: '12px',
                fontWeight: 600,
                border: '1px solid #22c55e',
                borderRadius: '6px',
                background: '#22c55e22',
                color: '#22c55e',
                cursor: 'pointer',
              }}
            >
              Approve & Merge
            </button>
            <button
              onClick={() => handleAction('discard')}
              style={{
                flex: 1,
                padding: '8px 12px',
                fontSize: '12px',
                fontWeight: 600,
                border: '1px solid #ef4444',
                borderRadius: '6px',
                background: '#ef444422',
                color: '#ef4444',
                cursor: 'pointer',
              }}
            >
              Discard
            </button>
          </>
        )}

        {(run.status === 'completed' || run.status === 'discarded' || run.status === 'stopped') && (
          <button
            onClick={onClose}
            style={{
              flex: 1,
              padding: '8px 12px',
              fontSize: '12px',
              fontWeight: 600,
              border: '1px solid #252535',
              borderRadius: '6px',
              background: 'transparent',
              color: '#555',
              cursor: 'pointer',
            }}
          >
            Close
          </button>
        )}
      </div>
    </div>
  );
}
