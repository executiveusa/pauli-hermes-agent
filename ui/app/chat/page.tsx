'use client';

import { useState, useRef, useEffect, useCallback } from 'react';
import { useStore } from '@/lib/store';
import FreeModeToggle from '@/components/FreeModeToggle';

const PROVIDER_LABELS: Record<string, string> = {
  synthia: '⚡ Groq/OpenAI',
  mercury: '💎 Mercury',
  'nvidia-nim': '🚀 NVIDIA',
  groq: '⚡ Groq',
};

export default function ChatPage() {
  const { messages, addMessage, clearMessages, freeMode, apiUrl } = useStore();
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, loading]);

  const send = useCallback(async (text: string) => {
    if (!text.trim() || loading) return;
    setInput('');
    setLoading(true);

    addMessage({ id: `u-${Date.now()}`, role: 'user', text, ts: Date.now() });

    try {
      const body: Record<string, unknown> = {
        message: text,
        agent_type: 'hermes',
        providers: freeMode ? { nvidia: true } : { mercury: true, nvidia: true },
      };

      const res = await fetch(apiUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });

      const data = await res.json();
      if (!res.ok) throw new Error(data.detail || data.error || `Error ${res.status}`);

      addMessage({
        id: `a-${Date.now()}`,
        role: 'agent',
        text: data.response || 'Done.',
        provider: data.provider,
        ts: Date.now(),
      });
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Unknown error';
      addMessage({ id: `e-${Date.now()}`, role: 'agent', text: `Error: ${msg}`, ts: Date.now() });
    } finally {
      setLoading(false);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [loading, freeMode, apiUrl, addMessage]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100dvh - 52px)', background: '#0d0d14' }}>
      {/* Top bar */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '10px 16px', background: '#16161f', borderBottom: '1px solid #252535', flexShrink: 0, gap: '8px' }}>
        <span style={{ fontSize: '14px', fontWeight: 600, color: '#ccc' }}>Chat</span>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <FreeModeToggle compact />
          {messages.length > 0 && (
            <button
              onClick={clearMessages}
              style={{ background: 'none', border: '1px solid #333', color: '#666', borderRadius: '6px', padding: '4px 10px', fontSize: '12px', cursor: 'pointer' }}
            >
              Clear
            </button>
          )}
        </div>
      </div>

      {/* Messages */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '16px 12px 8px', display: 'flex', flexDirection: 'column', gap: '8px', WebkitOverflowScrolling: 'touch' } as React.CSSProperties}>
        {messages.length === 0 && (
          <div style={{ textAlign: 'center', color: '#44445a', fontSize: '14px', marginTop: '60px' }}>
            Ask Hermes anything…
          </div>
        )}
        {messages.map((m) => (
          <div key={m.id} style={{ display: 'flex', justifyContent: m.role === 'user' ? 'flex-end' : 'flex-start' }}>
            <div style={{
              maxWidth: '78%',
              padding: '10px 14px',
              borderRadius: '18px',
              fontSize: '15px',
              lineHeight: 1.45,
              display: 'flex',
              flexDirection: 'column',
              gap: '5px',
              wordBreak: 'break-word',
              ...(m.role === 'user'
                ? { background: '#3a5f99', color: '#fff', borderBottomRightRadius: '5px' }
                : { background: '#1c1c2a', color: '#dde', border: '1px solid #2a2a40', borderBottomLeftRadius: '5px' }),
            }}>
              <span>{m.text}</span>
              {m.provider && (
                <span style={{ fontSize: '10px', color: '#666', alignSelf: 'flex-end' }}>
                  {PROVIDER_LABELS[m.provider] || m.provider}
                </span>
              )}
            </div>
          </div>
        ))}
        {loading && (
          <div style={{ display: 'flex', justifyContent: 'flex-start' }}>
            <div style={{ background: '#1c1c2a', border: '1px solid #2a2a40', borderBottomLeftRadius: '5px', borderRadius: '18px', padding: '10px 14px', color: '#888' }}>…</div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Input bar */}
      <div style={{ display: 'flex', gap: '8px', padding: '8px 12px', background: '#16161f', borderTop: '1px solid #252535', flexShrink: 0 }}>
        <input
          ref={inputRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && send(input)}
          placeholder="Type a message…"
          disabled={loading}
          style={{ flex: 1, padding: '11px 16px', borderRadius: '22px', border: '1px solid #2e2e42', background: '#21212e', color: '#eee', fontSize: '15px', outline: 'none' }}
        />
        <button
          onClick={() => send(input)}
          disabled={!input.trim() || loading}
          style={{
            width: '44px', height: '44px', borderRadius: '50%', border: 'none',
            background: '#3a5f99', color: '#fff', fontSize: '20px', cursor: 'pointer',
            flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'center',
            opacity: input.trim() && !loading ? 1 : 0.35,
            transition: 'opacity 0.15s',
          }}
        >
          ↑
        </button>
      </div>
    </div>
  );
}
