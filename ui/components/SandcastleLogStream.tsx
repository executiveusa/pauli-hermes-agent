'use client';

import { useState, useRef, useEffect } from 'react';
import { SandcastleEvent } from '@/lib/schemas/sandcastle';

interface SandcastleLogStreamProps {
  events: SandcastleEvent[];
  severityFilter?: ('error' | 'warning' | 'info' | 'debug')[];
  onFilterChange?: (filter: ('error' | 'warning' | 'info' | 'debug')[]) => void;
}

export default function SandcastleLogStream({
  events,
  severityFilter = ['error', 'warning', 'info'],
  onFilterChange,
}: SandcastleLogStreamProps) {
  const [follow, setFollow] = useState(true);
  const [search, setSearch] = useState('');
  const [selectedFilter, setSelectedFilter] = useState(severityFilter);
  const containerRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (follow && scrollRef.current) {
      scrollRef.current.scrollIntoView({ behavior: 'auto' });
    }
  }, [events, follow]);

  const handleFilterChange = (severity: 'error' | 'warning' | 'info' | 'debug') => {
    const newFilter = selectedFilter.includes(severity)
      ? selectedFilter.filter(s => s !== severity)
      : [...selectedFilter, severity];
    setSelectedFilter(newFilter);
    onFilterChange?.(newFilter);
  };

  const filtered = events.filter(e => {
    const matchesSeverity = selectedFilter.includes(e.severity as any);
    const matchesSearch = search === '' || e.message.toLowerCase().includes(search.toLowerCase());
    return matchesSeverity && matchesSearch;
  });

  const formatLogLine = (event: SandcastleEvent, lineNumber: number): string => {
    const time = new Date(event.timestamp).toLocaleTimeString();
    const type = event.type.padEnd(20);
    return `${lineNumber.toString().padStart(4)} ${time} [${event.severity.toUpperCase()}] ${type} ${event.message}`;
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', maxHeight: '400px', border: '1px solid #252535', borderRadius: '8px', overflow: 'hidden' }}>
      {/* Controls */}
      <div style={{ padding: '10px 12px', background: '#0d0d14', borderBottom: '1px solid #252535', display: 'flex', gap: '10px', flexWrap: 'wrap', alignItems: 'center' }}>
        <div style={{ display: 'flex', gap: '6px' }}>
          {(['error', 'warning', 'info'] as const).map(severity => (
            <button
              key={severity}
              onClick={() => handleFilterChange(severity)}
              style={{
                padding: '4px 10px',
                fontSize: '11px',
                fontWeight: 500,
                border: '1px solid ' + (selectedFilter.includes(severity) ? '#3a5f99' : '#252535'),
                borderRadius: '4px',
                background: selectedFilter.includes(severity) ? '#3a5f9944' : 'transparent',
                color: selectedFilter.includes(severity) ? '#7aa0cc' : '#555',
                cursor: 'pointer',
                transition: 'all 0.15s ease',
              }}
              onMouseEnter={(e) => {
                if (!selectedFilter.includes(severity)) {
                  e.currentTarget.style.borderColor = '#3a5f99';
                  e.currentTarget.style.color = '#7aa0cc';
                }
              }}
              onMouseLeave={(e) => {
                if (!selectedFilter.includes(severity)) {
                  e.currentTarget.style.borderColor = '#252535';
                  e.currentTarget.style.color = '#555';
                }
              }}
            >
              {severity.toUpperCase()}
            </button>
          ))}
        </div>

        <input
          type="text"
          placeholder="Search logs..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{
            flex: 1,
            minWidth: '150px',
            padding: '4px 8px',
            fontSize: '11px',
            border: '1px solid #252535',
            borderRadius: '4px',
            background: '#16161f',
            color: '#ccc',
            outline: 'none',
          }}
          onFocus={(e) => (e.target.style.borderColor = '#3a5f99')}
          onBlur={(e) => (e.target.style.borderColor = '#252535')}
        />

        <button
          onClick={() => setFollow(!follow)}
          style={{
            padding: '4px 10px',
            fontSize: '11px',
            fontWeight: 500,
            border: '1px solid ' + (follow ? '#22c55e' : '#252535'),
            borderRadius: '4px',
            background: follow ? '#22c55e22' : 'transparent',
            color: follow ? '#22c55e' : '#555',
            cursor: 'pointer',
            transition: 'all 0.15s ease',
          }}
          title="Follow new logs"
        >
          FOLLOW
        </button>

        <button
          onClick={() => {
            const text = filtered.map((e, i) => formatLogLine(e, i + 1)).join('\n');
            navigator.clipboard.writeText(text);
          }}
          style={{
            padding: '4px 10px',
            fontSize: '11px',
            fontWeight: 500,
            border: '1px solid #252535',
            borderRadius: '4px',
            background: 'transparent',
            color: '#555',
            cursor: 'pointer',
            transition: 'all 0.15s ease',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.borderColor = '#3a5f99';
            e.currentTarget.style.color = '#7aa0cc';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.borderColor = '#252535';
            e.currentTarget.style.color = '#555';
          }}
          title="Copy to clipboard"
        >
          COPY
        </button>
      </div>

      {/* Log output */}
      <div
        ref={containerRef}
        style={{
          flex: 1,
          overflow: 'auto',
          fontFamily: 'monospace',
          fontSize: '11px',
          lineHeight: '1.5',
          padding: '8px 12px',
          color: '#888',
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-word',
        }}
      >
        {filtered.length === 0 ? (
          <div style={{ color: '#555', padding: '20px', textAlign: 'center' }}>
            {events.length === 0 ? 'No logs' : 'No matching logs'}
          </div>
        ) : (
          <>
            {filtered.map((event, index) => (
              <div
                key={event.id}
                style={{
                  color:
                    event.severity === 'error'
                      ? '#ef4444'
                      : event.severity === 'warn'
                      ? '#eab308'
                      : '#888',
                }}
              >
                {formatLogLine(event, index + 1)}
              </div>
            ))}
            <div ref={scrollRef} />
          </>
        )}
      </div>

      {/* Stats */}
      <div
        style={{
          padding: '6px 12px',
          fontSize: '10px',
          color: '#555',
          borderTop: '1px solid #252535',
          background: '#0d0d14',
        }}
      >
        Showing {filtered.length} of {events.length} logs
      </div>
    </div>
  );
}
