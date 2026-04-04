// components/AgentStatus.tsx — Service health + pending approvals dashboard

import React, { useEffect, useState } from "react";
import { useSettingsStore } from "../stores/settings";
import * as bridge from "../lib/tauri-bridge";
import type { DashboardOverview, AgentRun, Approval } from "../lib/types";

export function AgentStatus() {
  const servicesStatus         = useSettingsStore((s) => s.servicesStatus);
  const refreshServicesStatus  = useSettingsStore((s) => s.refreshServicesStatus);

  const [overview, setOverview]     = useState<DashboardOverview | null>(null);
  const [runs, setRuns]             = useState<AgentRun[]>([]);
  const [approvals, setApprovals]   = useState<Approval[]>([]);
  const [loading, setLoading]       = useState(true);

  const loadData = async () => {
    setLoading(true);
    await refreshServicesStatus();
    try {
      const [ov, r, a] = await Promise.allSettled([
        bridge.getDashboardOverview(),
        bridge.getRuns(10),
        bridge.getApprovals(),
      ]);
      if (ov.status === "fulfilled") setOverview(ov.value);
      if (r.status === "fulfilled")  setRuns(r.value.runs ?? []);
      if (a.status === "fulfilled")  setApprovals(a.value.approvals ?? []);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 8000);
    return () => clearInterval(interval);
  }, []);

  const handleApprove = async (id: string) => {
    await bridge.approveCommand(id);
    setApprovals((prev) => prev.filter((a) => a.id !== id));
  };

  const handleReject = async (id: string) => {
    await bridge.rejectCommand(id);
    setApprovals((prev) => prev.filter((a) => a.id !== id));
  };

  const services = servicesStatus
    ? [
        servicesStatus.hermes_api,
        servicesStatus.llama_server,
        servicesStatus.whisper_server,
      ]
    : [];

  return (
    <div className="agent-panel">
      <div className="agent-panel__header">
        <h2 className="agent-panel__title">Agent Status</h2>
        <button className="btn btn-ghost" onClick={loadData} disabled={loading}>
          {loading ? "…" : "Refresh"}
        </button>
      </div>

      {/* Service health grid */}
      <section className="agent-panel__section">
        <h3 className="agent-panel__section-title">Services</h3>
        <div className="service-grid">
          {services.map((svc) => (
            <div
              key={svc.name}
              className={`service-card ${svc.healthy ? "service-card--online" : "service-card--offline"}`}
            >
              <div className="service-card__header">
                <span className={`status-dot ${svc.healthy ? "online" : "offline"}`} />
                <span className="service-card__name">{svc.name}</span>
                <span className={`badge ${svc.healthy ? "badge-success" : "badge-neutral"}`}>
                  {svc.healthy ? "running" : "stopped"}
                </span>
              </div>
              <span className="service-card__url mono">{svc.url}</span>
            </div>
          ))}
        </div>
      </section>

      {/* Overview metrics */}
      {overview && (
        <section className="agent-panel__section">
          <h3 className="agent-panel__section-title">Overview</h3>
          <div className="metrics-row">
            <MetricCard label="Total runs"   value={overview.total_runs} />
            <MetricCard label="Active runs"  value={overview.active_runs} accent={overview.active_runs > 0} />
            <MetricCard label="Pending approvals" value={overview.pending_approvals} accent={overview.pending_approvals > 0} warn />
            <MetricCard label="Tools loaded" value={overview.total_tools} />
          </div>
        </section>
      )}

      {/* Pending approvals */}
      {approvals.length > 0 && (
        <section className="agent-panel__section">
          <h3 className="agent-panel__section-title">
            Pending approvals
            <span className="badge badge-warning" style={{ marginLeft: 8 }}>{approvals.length}</span>
          </h3>
          <div className="approval-list">
            {approvals.map((ap) => (
              <div key={ap.id} className="approval-card">
                <div className="approval-card__header">
                  <code className="approval-card__tool">{ap.tool_name}</code>
                  <span className="text-secondary text-xs">{ap.reason}</span>
                </div>
                <pre className="approval-card__input">
                  {JSON.stringify(ap.tool_input, null, 2)}
                </pre>
                <div className="approval-card__actions">
                  <button
                    className="btn btn-primary"
                    onClick={() => handleApprove(ap.id)}
                  >
                    Approve
                  </button>
                  <button
                    className="btn btn-danger"
                    onClick={() => handleReject(ap.id)}
                  >
                    Reject
                  </button>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Recent runs */}
      {runs.length > 0 && (
        <section className="agent-panel__section">
          <h3 className="agent-panel__section-title">Recent runs</h3>
          <table className="runs-table">
            <thead>
              <tr>
                <th>Task</th>
                <th>Model</th>
                <th>Status</th>
                <th>Tools</th>
                <th>Started</th>
              </tr>
            </thead>
            <tbody>
              {runs.map((run) => (
                <tr key={run.id}>
                  <td className="runs-table__task truncate">{run.task}</td>
                  <td className="mono text-xs">{run.model}</td>
                  <td>
                    <span className={`badge ${statusBadge(run.status)}`}>{run.status}</span>
                  </td>
                  <td>{run.tool_calls}</td>
                  <td className="text-xs text-secondary">{fmtDate(run.started_at)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      )}

      <style>{agentStyles}</style>
    </div>
  );
}

function MetricCard({ label, value, accent, warn }: {
  label:   string;
  value:   number;
  accent?: boolean;
  warn?:   boolean;
}) {
  return (
    <div className={`metric-card ${accent && warn ? "metric-card--warn" : accent ? "metric-card--accent" : ""}`}>
      <span className="metric-card__value">{value}</span>
      <span className="metric-card__label">{label}</span>
    </div>
  );
}

function statusBadge(status: string) {
  switch (status) {
    case "completed": return "badge-success";
    case "running":   return "badge-info";
    case "failed":    return "badge-error";
    default:          return "badge-neutral";
  }
}

function fmtDate(iso: string) {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

const agentStyles = `
  .agent-panel {
    display: flex;
    flex-direction: column;
    gap: var(--sp-6);
    padding: var(--sp-6);
    overflow-y: auto;
    height: 100%;
    max-width: 900px;
    margin: 0 auto;
    width: 100%;
  }

  .agent-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .agent-panel__title {
    font-size: var(--text-xl);
    font-weight: var(--weight-semi);
  }

  .agent-panel__section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .agent-panel__section-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    display: flex;
    align-items: center;
  }

  /* Services */
  .service-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--sp-3);
  }

  .service-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--sp-3) var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .service-card--online  { border-left: 3px solid var(--success); }
  .service-card--offline { border-left: 3px solid var(--text-tertiary); }

  .service-card__header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .service-card__name {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    flex: 1;
  }

  .service-card__url {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }

  /* Metrics */
  .metrics-row {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: var(--sp-3);
  }

  .metric-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .metric-card--accent { border-color: var(--accent); }
  .metric-card--warn   { border-color: var(--warning); }

  .metric-card__value {
    font-size: var(--text-2xl);
    font-weight: var(--weight-semi);
    font-variant-numeric: tabular-nums;
  }

  .metric-card__label {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  /* Approvals */
  .approval-list {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .approval-card {
    background: var(--surface);
    border: 1px solid rgba(245,158,11,0.3);
    border-radius: var(--radius);
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .approval-card__header {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .approval-card__tool {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--warning);
  }

  .approval-card__input {
    font-size: var(--text-xs);
    max-height: 120px;
    overflow-y: auto;
  }

  .approval-card__actions {
    display: flex;
    gap: var(--sp-2);
  }

  /* Runs table */
  .runs-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .runs-table th {
    text-align: left;
    padding: var(--sp-2) var(--sp-3);
    color: var(--text-tertiary);
    font-weight: var(--weight-medium);
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-bottom: 1px solid var(--border);
  }

  .runs-table td {
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
    max-width: 260px;
  }

  .runs-table tr:last-child td { border-bottom: none; }
  .runs-table tr:hover td { background: var(--surface-2); }
`;
