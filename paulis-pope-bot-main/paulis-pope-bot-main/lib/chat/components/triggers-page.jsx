'use client';

import { useState, useEffect } from 'react';
import { ZapIcon, ChevronDownIcon } from './icons.js';
import { getSwarmConfig } from '../actions.js';

const typeBadgeStyles = {
  agent: 'bg-purple-500/10 text-purple-500',
  command: 'bg-blue-500/10 text-blue-500',
  webhook: 'bg-orange-500/10 text-orange-500',
};

const typeOrder = { agent: 0, command: 1, webhook: 2 };

function sortByType(items) {
  return [...items].sort((a, b) => {
    const actions_a = a.actions || [];
    const actions_b = b.actions || [];
    const ta = typeOrder[(actions_a[0]?.type) || 'agent'] ?? 99;
    const tb = typeOrder[(actions_b[0]?.type) || 'agent'] ?? 99;
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
// Action Card (nested inside trigger)
// ─────────────────────────────────────────────────────────────────────────────

function ActionCard({ action, index }) {
  const type = action.type || 'agent';

  return (
    <div className="rounded-md border bg-background p-3">
      <div className="flex items-center gap-2 mb-2">
        <span className="text-xs text-muted-foreground font-medium">Action {index + 1}</span>
        <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-medium ${typeBadgeStyles[type] || typeBadgeStyles.agent}`}>
          {type}
        </span>
      </div>
      {type === 'agent' && action.job && (
        <pre className="text-xs bg-muted rounded-md p-3 whitespace-pre-wrap break-words font-mono overflow-auto max-h-48">
          {action.job}
        </pre>
      )}
      {type === 'command' && action.command && (
        <pre className="text-xs bg-muted rounded-md p-3 whitespace-pre-wrap break-words font-mono overflow-auto max-h-48">
          {action.command}
        </pre>
      )}
      {type === 'webhook' && (
        <div className="flex flex-col gap-2">
          <pre className="text-xs bg-muted rounded-md p-3 whitespace-pre-wrap break-words font-mono overflow-auto">
            {action.method && action.method !== 'POST' ? `${action.method} ` : ''}{action.url}
          </pre>
          {action.vars && Object.keys(action.vars).length > 0 && (
            <div>
              <p className="text-xs font-medium text-muted-foreground mb-1">Variables</p>
              <pre className="text-xs bg-muted rounded-md p-3 whitespace-pre-wrap break-words font-mono overflow-auto max-h-48">
                {JSON.stringify(action.vars, null, 2)}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Trigger Card
// ─────────────────────────────────────────────────────────────────────────────

function TriggerCard({ trigger }) {
  const [expanded, setExpanded] = useState(false);
  const disabled = trigger.enabled === false;
  const actions = trigger.actions || [];
  const actionTypes = actions
    .map((a) => a.type || 'agent')
    .filter((v, i, arr) => arr.indexOf(v) === i);

  return (
    <div
      className={`rounded-lg border bg-card transition-opacity ${disabled ? 'opacity-60' : ''}`}
    >
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-3 w-full text-left p-4 hover:bg-accent/50 rounded-lg"
      >
        <div className="shrink-0 rounded-md bg-muted p-2">
          <ZapIcon size={16} />
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium truncate">{trigger.name}</p>
          <p className="text-xs text-muted-foreground mt-0.5">
            <span className="font-mono">{trigger.watch_path}</span>
            <span className="mx-1.5 text-border">|</span>
            {actions.length} action{actions.length !== 1 ? 's' : ''}
            {actionTypes.length > 0 && (
              <span className="ml-1">({actionTypes.join(', ')})</span>
            )}
          </p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
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
        <div className="border-t px-4 py-3 flex flex-col gap-2">
          {actions.length === 0 ? (
            <p className="text-xs text-muted-foreground">No actions defined.</p>
          ) : (
            actions.map((action, i) => (
              <ActionCard key={i} action={action} index={i} />
            ))
          )}
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Main Page
// ─────────────────────────────────────────────────────────────────────────────

export function TriggersPage() {
  const [triggers, setTriggers] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getSwarmConfig()
      .then((data) => {
        if (data?.triggers) setTriggers(data.triggers);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const enabled = sortByType(triggers.filter((t) => t.enabled !== false));
  const disabled = sortByType(triggers.filter((t) => t.enabled === false));

  return (
    <>
      {!loading && (
        <p className="text-sm text-muted-foreground mb-4">
          {triggers.length} trigger{triggers.length !== 1 ? 's' : ''} configured, {enabled.length} enabled
        </p>
      )}

      {loading ? (
        <div className="flex flex-col gap-3">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-20 animate-pulse rounded-lg bg-border/50" />
          ))}
        </div>
      ) : triggers.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <div className="rounded-full bg-muted p-4 mb-4">
            <ZapIcon size={24} />
          </div>
          <p className="text-sm font-medium mb-1">No triggers configured</p>
          <p className="text-xs text-muted-foreground max-w-sm">
            Add webhook triggers by editing <span className="font-mono">config/TRIGGERS.json</span> in your project.
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          {enabled.length > 0 && (
            <>
              <GroupHeader label="Enabled" count={enabled.length} />
              {enabled.map((trigger, i) => (
                <TriggerCard key={`enabled-${i}`} trigger={trigger} />
              ))}
            </>
          )}
          {disabled.length > 0 && (
            <>
              <GroupHeader label="Disabled" count={disabled.length} />
              {disabled.map((trigger, i) => (
                <TriggerCard key={`disabled-${i}`} trigger={trigger} />
              ))}
            </>
          )}
        </div>
      )}
    </>
  );
}
