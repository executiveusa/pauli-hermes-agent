'use client';

import { useState, useEffect } from 'react';
import { ClockIcon, SpinnerIcon, ChevronDownIcon } from './icons.js';
import { getSwarmConfig } from '../actions.js';

// ─────────────────────────────────────────────────────────────────────────────
// Utilities
// ─────────────────────────────────────────────────────────────────────────────

function describeCron(schedule) {
  const parts = schedule.trim().split(/\s+/);
  if (parts.length !== 5) return schedule;

  const [minute, hour, dayOfMonth, month, dayOfWeek] = parts;

  // Every N minutes
  if (minute.startsWith('*/') && hour === '*' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
    const n = parseInt(minute.slice(2), 10);
    if (n === 1) return 'Every minute';
    return `Every ${n} minutes`;
  }

  // Every N hours
  if (hour.startsWith('*/') && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
    const n = parseInt(hour.slice(2), 10);
    if (n === 1) return 'Every hour';
    return `Every ${n} hours`;
  }

  // Specific time daily
  if (minute !== '*' && hour !== '*' && !hour.includes('/') && !minute.includes('/') && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
    const h = parseInt(hour, 10);
    const m = parseInt(minute, 10);
    const period = h >= 12 ? 'PM' : 'AM';
    const displayH = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return `Daily at ${displayH}:${String(m).padStart(2, '0')} ${period}`;
  }

  // Specific time on specific weekdays
  if (minute !== '*' && hour !== '*' && dayOfMonth === '*' && month === '*' && dayOfWeek !== '*') {
    const dayNames = { '0': 'Sun', '1': 'Mon', '2': 'Tue', '3': 'Wed', '4': 'Thu', '5': 'Fri', '6': 'Sat' };
    const days = dayOfWeek.split(',').map(d => dayNames[d] || d).join(', ');
    const h = parseInt(hour, 10);
    const m = parseInt(minute, 10);
    const period = h >= 12 ? 'PM' : 'AM';
    const displayH = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return `${days} at ${displayH}:${String(m).padStart(2, '0')} ${period}`;
  }

  return schedule;
}

const typeBadgeStyles = {
  agent: 'bg-purple-500/10 text-purple-500',
  command: 'bg-blue-500/10 text-blue-500',
  webhook: 'bg-orange-500/10 text-orange-500',
};

const typeOrder = { agent: 0, command: 1, webhook: 2 };

function sortByType(items) {
  return [...items].sort((a, b) => {
    const ta = typeOrder[a.type || 'agent'] ?? 99;
    const tb = typeOrder[b.type || 'agent'] ?? 99;
    return ta - tb;
  });
}

// ─────────────────────────────────────────────────────────────────────────────
// Group Header
// ─────────────────────────────────────────────────────────────────────────────

function GroupHeader({ label, count }) {
  return (
    <div className="flex items-center gap-2 pt-2 pb-1">
      <span className="text-xs font-medium text-muted-foreground uppercase tracking-wide">{label}</span>
      <span className="text-xs text-muted-foreground">({count})</span>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Cron Card
// ─────────────────────────────────────────────────────────────────────────────

function CronCard({ cron }) {
  const [expanded, setExpanded] = useState(false);
  const type = cron.type || 'agent';
  const disabled = cron.enabled === false;

  return (
    <div
      className={`rounded-lg border bg-card transition-opacity ${disabled ? 'opacity-60' : ''}`}
    >
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-3 w-full text-left p-4 hover:bg-accent/50 rounded-lg"
      >
        <div className="shrink-0 rounded-md bg-muted p-2">
          <ClockIcon size={16} />
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium truncate">{cron.name}</p>
          <p className="text-xs text-muted-foreground mt-0.5">
            <span className="font-mono">{cron.schedule}</span>
            <span className="mx-1.5 text-border">|</span>
            {describeCron(cron.schedule)}
          </p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-medium ${typeBadgeStyles[type] || typeBadgeStyles.agent}`}>
            {type}
          </span>
          <span
            className={`inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-medium ${
              disabled ? 'bg-muted text-muted-foreground' : 'bg-green-500/10 text-green-500'
            }`}
          >
            {disabled ? 'disabled' : 'enabled'}
          </span>
          <span className={`transition-transform ${expanded ? 'rotate-180' : ''}`}>
            <ChevronDownIcon size={14} />
          </span>
        </div>
      </button>

      {expanded && (
        <div className="border-t px-4 py-3">
          {type === 'agent' && cron.job && (
            <div>
              <p className="text-xs font-medium text-muted-foreground mb-1.5">Job prompt</p>
              <pre className="text-xs bg-muted rounded-md p-3 whitespace-pre-wrap break-words font-mono overflow-auto max-h-48">
                {cron.job}
              </pre>
            </div>
          )}
          {type === 'command' && cron.command && (
            <div>
              <p className="text-xs font-medium text-muted-foreground mb-1.5">Command</p>
              <pre className="text-xs bg-muted rounded-md p-3 whitespace-pre-wrap break-words font-mono overflow-auto max-h-48">
                {cron.command}
              </pre>
            </div>
          )}
          {type === 'webhook' && (
            <div className="flex flex-col gap-2">
              <div>
                <p className="text-xs font-medium text-muted-foreground mb-1.5">URL</p>
                <pre className="text-xs bg-muted rounded-md p-3 whitespace-pre-wrap break-words font-mono overflow-auto">
                  {cron.method && cron.method !== 'POST' ? `${cron.method} ` : ''}{cron.url}
                </pre>
              </div>
              {cron.vars && Object.keys(cron.vars).length > 0 && (
                <div>
                  <p className="text-xs font-medium text-muted-foreground mb-1.5">Variables</p>
                  <pre className="text-xs bg-muted rounded-md p-3 whitespace-pre-wrap break-words font-mono overflow-auto max-h-48">
                    {JSON.stringify(cron.vars, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Main Page
// ─────────────────────────────────────────────────────────────────────────────

export function CronsPage() {
  const [crons, setCrons] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getSwarmConfig()
      .then((data) => {
        if (data?.crons) setCrons(data.crons);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const enabled = sortByType(crons.filter((c) => c.enabled !== false));
  const disabled = sortByType(crons.filter((c) => c.enabled === false));

  return (
    <>
      {!loading && (
        <p className="text-sm text-muted-foreground mb-4">
          {crons.length} job{crons.length !== 1 ? 's' : ''} configured, {enabled.length} enabled
        </p>
      )}

      {loading ? (
        <div className="flex flex-col gap-3">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-20 animate-pulse rounded-lg bg-border/50" />
          ))}
        </div>
      ) : crons.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <div className="rounded-full bg-muted p-4 mb-4">
            <ClockIcon size={24} />
          </div>
          <p className="text-sm font-medium mb-1">No cron jobs configured</p>
          <p className="text-xs text-muted-foreground max-w-sm">
            Add scheduled jobs by editing <span className="font-mono">config/CRONS.json</span> in your project.
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          {enabled.length > 0 && (
            <>
              <GroupHeader label="Enabled" count={enabled.length} />
              {enabled.map((cron, i) => (
                <CronCard key={`enabled-${i}`} cron={cron} />
              ))}
            </>
          )}
          {disabled.length > 0 && (
            <>
              <GroupHeader label="Disabled" count={disabled.length} />
              {disabled.map((cron, i) => (
                <CronCard key={`disabled-${i}`} cron={cron} />
              ))}
            </>
          )}
        </div>
      )}
    </>
  );
}
