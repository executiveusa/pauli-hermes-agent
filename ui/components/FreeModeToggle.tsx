'use client';

import { useStore } from '@/lib/store';

export default function FreeModeToggle({ compact = false }: { compact?: boolean }) {
  const { freeMode, setFreeMode } = useStore();

  return (
    <button
      onClick={() => setFreeMode(!freeMode)}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        padding: compact ? '4px 10px' : '6px 14px',
        borderRadius: '20px',
        border: `1px solid ${freeMode ? '#22c55e' : '#3a5f99'}`,
        background: freeMode ? 'rgba(34,197,94,0.12)' : 'rgba(58,95,153,0.12)',
        color: freeMode ? '#22c55e' : '#7aa0cc',
        cursor: 'pointer',
        fontSize: compact ? '11px' : '12px',
        fontWeight: 600,
        letterSpacing: '0.04em',
        transition: 'all 0.2s ease',
        whiteSpace: 'nowrap',
      }}
      title={freeMode ? 'FREE MODE: routing to NVIDIA NIM (free)' : 'Click to enable free NVIDIA inference'}
    >
      <span style={{ fontSize: compact ? '12px' : '14px' }}>{freeMode ? '⚡' : '💠'}</span>
      {freeMode ? 'FREE' : 'Premium'}
    </button>
  );
}
