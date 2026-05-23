'use client';

import { useState, useRef, useEffect } from 'react';
import { SandcastleEvent } from '@/lib/schemas/sandcastle';

interface SandcastleEventTimelineProps {
  events: SandcastleEvent[];
  loading?: boolean;
  onEventClick?: (event: SandcastleEvent) => void;
}

const EVENT_ICONS: Record<string, string> = {
  provider_selected: '🎯',
  branch_created: '🌿',
  sandbox_created: '📦',
  prompt_loaded: '📝',
  agent_started: '🤖',
  agent_stream_text: '💬',
  tool_call: '🔧',
  file_changed: '📄',
  command_started: '▶️',
  command_finished: '✓',
  test_started: '🧪',
  test_finished: '✅',
  commit_created: '💾',
  review_started: '👀',
  review_finished: '📋',
  approval_required: '⏳',
  merge_approved: '🚀',
  sandbox_discarded: '🗑️',
  error: '❌',
};

const EVENT_COLORS: Record<string, string> = {
  provider_selected: '#3a5f99',
  branch_created: '#0d7377',
  sandbox_created: '#22c55e',
  agent_started: '#22c55e',
  test_finished: '#a855f7',
  commit_created: '#f97316',
  approval_required: '#eab308',
  merge_approved: '#22c55e',
  error: '#ef4444',
};

export default function SandcastleEventTimeline({
  events,
  loading = false,
  onEventClick,
}: SandcastleEventTimelineProps) {
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current && events.length > 0) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [events]);

  const errorCount = events.filter(e => e.severity === 'error').length;
  const warnCount = events.filter(e => e.severity === 'warn').length;
  const infoCount = events.filter(e => e.severity === 'info').length;

  const formatTime = (timestamp: number) => {
    const now = Date.now();
    const diff = now - timestamp;
    const minutes = Math.floor(diff / 60000);
    const seconds = Math.floor((diff % 60000) / 1000);

    if (minutes > 0) return `${minutes}m ago`;
    if (seconds > 0) return `${seconds}s ago`;
    return 'now';
  };

  if (loading) {
    return (
      <div style={{ padding: '24px', textAlign: 'center', color: '#555' }}>
        Loading events...
      </div>
    );
  }

  if (events.length === 0) {
    return (
      <div style={{ padding: '24px', textAlign: 'center', color: '#555' }}>
        No events yet
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div
        style={{
          padding: '12px 16px',
          borderBottom: '1px solid #252535',
          display: 'flex',
          alignItems: 'center',
          gap: '12px',
          fontSize: '12px',
          color: '#777',
        }}
      >
        <span>{events.length} events</span>
        {errorCount > 0 && <span>• ❌ {errorCount} error</span>}
        {warnCount > 0 && <span>• ⚠️ {warnCount} warning</span>}
        {infoCount > 0 && <span>• ℹ️ {infoCount} info</span>}
      </div>

      <div
        ref={containerRef}
        style={{
          flex: 1,
          overflow: 'auto',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {events.map((event, index) => {
          const isExpanded = expandedId === event.id;
          const color = EVENT_COLORS[event.type] || '#666';
          const icon = EVENT_ICONS[event.type] || '•';

          return (
            <div
              key={event.id}
              style={{
                padding: '12px 16px',
                borderBottom: index < events.length - 1 ? '1px solid #1a1a28' : 'none',
                display: 'flex',
                gap: '10px',
                cursor: onEventClick ? 'pointer' : 'default',
                transition: 'background 0.15s ease',
              }}
              onClick={() => {
                if (onEventClick) onEventClick(event);
                setExpandedId(isExpanded ? null : event.id);
              }}
              onMouseEnter={(e) => {
                if (onEventClick) {
                  e.currentTarget.style.background = '#1a1a28';
                }
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'transparent';
              }}
            >
              <div
                style={{
                  fontSize: '14px',
                  minWidth: '20px',
                  marginTop: '2px',
                  opacity: 0.8,
                }}
              >
                {icon}
              </div>

              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '2px' }}>
                  <span
                    style={{
                      fontSize: '12px',
                      fontWeight: 600,
                      color,
                    }}
                  >
                    {event.type.replace(/_/g, ' ')}
                  </span>
                  <span
                    style={{
                      fontSize: '11px',
                      color: '#555',
                    }}
                    title={new Date(event.timestamp).toISOString()}
                  >
                    {formatTime(event.timestamp)}
                  </span>
                  {event.severity !== 'info' && (
                    <span
                      style={{
                        fontSize: '10px',
                        padding: '2px 6px',
                        borderRadius: '4px',
                        background:
                          event.severity === 'error' ? '#ef444433' :
                          event.severity === 'warn' ? '#eab30833' : 'transparent',
                        color:
                          event.severity === 'error' ? '#ef4444' :
                          event.severity === 'warn' ? '#eab308' : '#555',
                      }}
                    >
                      {event.severity.toUpperCase()}
                    </span>
                  )}
                </div>

                <div style={{ fontSize: '13px', color: '#ccc', wordBreak: 'break-word' }}>
                  {event.message}
                </div>

                {isExpanded && event.metadata && Object.keys(event.metadata).length > 0 && (
                  <div
                    style={{
                      marginTop: '8px',
                      padding: '8px 10px',
                      background: '#0d0d14',
                      borderRadius: '6px',
                      fontSize: '11px',
                      color: '#888',
                      fontFamily: 'monospace',
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-all',
                    }}
                  >
                    {JSON.stringify(event.metadata, null, 2)}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
