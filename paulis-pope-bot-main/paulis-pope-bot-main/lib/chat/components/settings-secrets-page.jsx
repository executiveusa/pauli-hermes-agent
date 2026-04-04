'use client';

import { useState, useEffect } from 'react';
import { KeyIcon, CopyIcon, CheckIcon, TrashIcon, RefreshIcon } from './icons.js';
import { createNewApiKey, getApiKeys, deleteApiKey } from '../actions.js';

function timeAgo(ts) {
  if (!ts) return 'Never';
  const seconds = Math.floor((Date.now() - ts) / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  const months = Math.floor(days / 30);
  return `${months}mo ago`;
}

function formatDate(ts) {
  if (!ts) return '—';
  return new Date(ts).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function CopyButton({ text }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      const textarea = document.createElement('textarea');
      textarea.value = text;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <button
      onClick={handleCopy}
      className="inline-flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium border border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground"
    >
      {copied ? <CheckIcon size={14} /> : <CopyIcon size={14} />}
      {copied ? 'Copied' : 'Copy'}
    </button>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Section wrapper — reusable for each secrets section
// ─────────────────────────────────────────────────────────────────────────────

function Section({ title, description, children }) {
  return (
    <div className="pb-8 mb-8 border-b border-border last:border-b-0 last:pb-0 last:mb-0">
      <h2 className="text-base font-medium mb-1">{title}</h2>
      {description && (
        <p className="text-sm text-muted-foreground mb-4">{description}</p>
      )}
      {children}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// API Key section
// ─────────────────────────────────────────────────────────────────────────────

function ApiKeySection() {
  const [currentKey, setCurrentKey] = useState(null);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [newKey, setNewKey] = useState(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [confirmRegenerate, setConfirmRegenerate] = useState(false);
  const [error, setError] = useState(null);

  const loadKey = async () => {
    try {
      const result = await getApiKeys();
      setCurrentKey(result);
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadKey();
  }, []);

  const handleCreate = async () => {
    if (creating) return;
    setCreating(true);
    setError(null);
    setConfirmRegenerate(false);
    try {
      const result = await createNewApiKey();
      if (result.error) {
        setError(result.error);
      } else {
        setNewKey(result.key);
        await loadKey();
      }
    } catch {
      setError('Failed to create API key');
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async () => {
    if (!confirmDelete) {
      setConfirmDelete(true);
      setTimeout(() => setConfirmDelete(false), 3000);
      return;
    }
    try {
      await deleteApiKey();
      setCurrentKey(null);
      setNewKey(null);
      setConfirmDelete(false);
    } catch {
      // ignore
    }
  };

  const handleRegenerate = () => {
    if (!confirmRegenerate) {
      setConfirmRegenerate(true);
      setTimeout(() => setConfirmRegenerate(false), 3000);
      return;
    }
    handleCreate();
  };

  if (loading) {
    return <div className="h-14 animate-pulse rounded-md bg-border/50" />;
  }

  return (
    <div>
      {error && (
        <p className="text-sm text-destructive mb-4">{error}</p>
      )}

      {/* New key banner */}
      {newKey && (
        <div className="rounded-lg border border-green-500/30 bg-green-500/5 p-4 mb-4">
          <div className="flex items-start justify-between gap-3 mb-2">
            <p className="text-sm font-medium text-green-600 dark:text-green-400">
              API key created — copy it now. You won't be able to see it again.
            </p>
            <button
              onClick={() => setNewKey(null)}
              className="text-xs text-muted-foreground hover:text-foreground shrink-0"
            >
              Dismiss
            </button>
          </div>
          <div className="flex items-center gap-2">
            <code className="flex-1 rounded-md bg-muted px-3 py-2 text-xs font-mono break-all select-all">
              {newKey}
            </code>
            <CopyButton text={newKey} />
          </div>
        </div>
      )}

      {currentKey ? (
        <div className="rounded-lg border bg-card p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="shrink-0 rounded-md bg-muted p-2">
                <KeyIcon size={16} />
              </div>
              <div>
                <code className="text-sm font-mono">{currentKey.keyPrefix}...</code>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Created {formatDate(currentKey.createdAt)}
                  {currentKey.lastUsedAt && (
                    <span className="ml-2">· Last used {timeAgo(currentKey.lastUsedAt)}</span>
                  )}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={handleRegenerate}
                disabled={creating}
                className={`inline-flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium border ${
                  confirmRegenerate
                    ? 'border-yellow-500 text-yellow-600 hover:bg-yellow-500/10'
                    : 'border-border text-muted-foreground hover:bg-accent hover:text-foreground'
                } disabled:opacity-50`}
              >
                <RefreshIcon size={12} />
                {creating ? 'Generating...' : confirmRegenerate ? 'Confirm regenerate' : 'Regenerate'}
              </button>
              <button
                onClick={handleDelete}
                className={`inline-flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium border ${
                  confirmDelete
                    ? 'border-destructive text-destructive hover:bg-destructive/10'
                    : 'border-border text-muted-foreground hover:text-destructive hover:border-destructive/50'
                }`}
              >
                <TrashIcon size={12} />
                {confirmDelete ? 'Confirm delete' : 'Delete'}
              </button>
            </div>
          </div>
        </div>
      ) : (
        <div className="rounded-lg border border-dashed bg-card p-6 flex flex-col items-center text-center">
          <p className="text-sm text-muted-foreground mb-3">No API key configured</p>
          <button
            onClick={handleCreate}
            disabled={creating}
            className="inline-flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium bg-foreground text-background hover:bg-foreground/90 disabled:opacity-50 disabled:pointer-events-none"
          >
            {creating ? 'Creating...' : 'Create API key'}
          </button>
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Main page
// ─────────────────────────────────────────────────────────────────────────────

export function SettingsSecretsPage() {
  return (
    <div>
      <Section
        title="API Key"
        description="Authenticates external requests to /api endpoints. Pass via the x-api-key header."
      >
        <ApiKeySection />
      </Section>

      {/* Future sections go here, e.g.:
      <Section title="GitHub Token" description="...">
        <GitHubTokenSection />
      </Section>
      */}
    </div>
  );
}
