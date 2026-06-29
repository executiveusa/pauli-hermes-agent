'use client';

import { useState, useRef, useEffect, useCallback } from 'react';
import { useStore } from '@/lib/store';
import FreeModeToggle from '@/components/FreeModeToggle';

const PROVIDER_LABELS: Record<string, string> = {
  synthia: '⚡ Groq/OpenAI',
  mercury: '💎 Mercury',
  'nvidia-nim': '🚀 NVIDIA',
};

interface Msg { id: string; role: 'user' | 'agent'; text: string; provider?: string; }

export default function VoicePage() {
  const { freeMode, apiUrl } = useStore();
  const [msgs, setMsgs] = useState<Msg[]>([]);
  const [isListening, setIsListening] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [inputText, setInputText] = useState('');
  const [interim, setInterim] = useState('');
  const recRef = useRef<any>(null);
  const pendingRef = useRef('');
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [msgs, interim]);

  useEffect(() => {
    const Ctor = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!Ctor) return;
    const rec = new Ctor();
    rec.continuous = false;
    rec.interimResults = true;
    rec.lang = 'en-US';
    rec.onstart = () => setIsListening(true);
    rec.onresult = (event: any) => {
      let final = '', inter = '';
      for (let i = event.resultIndex; i < event.results.length; i++) {
        if (event.results[i].isFinal) final += event.results[i][0].transcript;
        else inter += event.results[i][0].transcript;
      }
      if (final) pendingRef.current += final + ' ';
      setInterim(inter);
    };
    rec.onend = () => {
      setIsListening(false);
      setInterim('');
      const text = pendingRef.current.trim();
      pendingRef.current = '';
      if (text) send(text);
    };
    recRef.current = rec;
  }, []);

  const speak = (text: string) => {
    setIsSpeaking(true);
    const clean = text.replace(/^[💎🚀⚡]\s*/, '').substring(0, 300);
    const utt = new SpeechSynthesisUtterance(clean);
    utt.rate = 1.0;
    utt.onend = () => setIsSpeaking(false);
    utt.onerror = () => setIsSpeaking(false);
    window.speechSynthesis.cancel();
    window.speechSynthesis.speak(utt);
  };

  const send = useCallback(async (text: string) => {
    if (!text || isProcessing) return;
    setInputText('');
    setMsgs((prev) => [...prev, { id: `u-${Date.now()}`, role: 'user', text }]);
    setIsProcessing(true);
    try {
      const res = await fetch(apiUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: text,
          agent_type: 'hermes',
          providers: freeMode ? { nvidia: true } : { mercury: true, nvidia: true },
        }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data.detail || data.error || `Error ${res.status}`);
      const reply = data.response || 'Done.';
      setMsgs((prev) => [...prev, { id: `a-${Date.now()}`, role: 'agent', text: reply, provider: data.provider }]);
      speak(reply);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Unknown error';
      setMsgs((prev) => [...prev, { id: `e-${Date.now()}`, role: 'agent', text: `Error: ${msg}` }]);
    } finally {
      setIsProcessing(false);
    }
  }, [isProcessing, freeMode, apiUrl]);

  const startListening = (e: React.MouseEvent | React.TouchEvent) => {
    e.preventDefault();
    if (recRef.current && !isListening && !isProcessing) {
      pendingRef.current = '';
      recRef.current.start();
    }
  };
  const stopListening = (e: React.MouseEvent | React.TouchEvent) => {
    e.preventDefault();
    recRef.current?.stop();
  };

  const status = isListening ? '🎙️ Listening…' : isProcessing ? '⏳ Thinking…' : isSpeaking ? '🔊 Speaking…' : 'Hermes Voice';

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100dvh - 52px)', background: '#0d0d14' }}>
      {/* Top bar */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '10px 16px', background: '#16161f', borderBottom: '1px solid #252535', flexShrink: 0 }}>
        <span style={{ fontSize: '15px', fontWeight: 600, color: '#ccc' }}>{status}</span>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <FreeModeToggle compact />
          {msgs.length > 0 && (
            <button
              onClick={() => { setMsgs([]); window.speechSynthesis.cancel(); }}
              style={{ background: 'none', border: '1px solid #333', color: '#666', borderRadius: '6px', padding: '4px 10px', fontSize: '12px', cursor: 'pointer' }}
            >
              Clear
            </button>
          )}
        </div>
      </div>

      {/* Feed */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '16px 12px 8px', display: 'flex', flexDirection: 'column', gap: '8px' } as React.CSSProperties}>
        {msgs.length === 0 && !isListening && (
          <div style={{ textAlign: 'center', color: '#44445a', fontSize: '14px', marginTop: '60px' }}>Hold the mic and speak a command</div>
        )}
        {msgs.map((m) => (
          <div key={m.id} style={{ display: 'flex', justifyContent: m.role === 'user' ? 'flex-end' : 'flex-start' }}>
            <div style={{
              maxWidth: '78%', padding: '10px 14px', borderRadius: '18px', fontSize: '15px', lineHeight: 1.45,
              display: 'flex', flexDirection: 'column', gap: '5px', wordBreak: 'break-word',
              ...(m.role === 'user'
                ? { background: '#3a5f99', color: '#fff', borderBottomRightRadius: '5px' }
                : { background: '#1c1c2a', color: '#dde', border: '1px solid #2a2a40', borderBottomLeftRadius: '5px' }),
            }}>
              <span>{m.text}</span>
              {m.provider && <span style={{ fontSize: '10px', color: '#666', alignSelf: 'flex-end' }}>{PROVIDER_LABELS[m.provider] || m.provider}</span>}
            </div>
          </div>
        ))}
        {isListening && interim && (
          <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
            <div style={{ background: '#3a5f99', color: '#fff', borderBottomRightRadius: '5px', borderRadius: '18px', padding: '10px 14px', opacity: 0.55, wordBreak: 'break-word', maxWidth: '78%' }}>{interim}</div>
          </div>
        )}
        {isProcessing && (
          <div style={{ display: 'flex', justifyContent: 'flex-start' }}>
            <div style={{ background: '#1c1c2a', border: '1px solid #2a2a40', borderBottomLeftRadius: '5px', borderRadius: '18px', padding: '10px 14px', color: '#888' }}>…</div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Text input bar */}
      <div style={{ display: 'flex', gap: '8px', padding: '8px 12px', background: '#16161f', borderTop: '1px solid #252535', flexShrink: 0 }}>
        <input
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && inputText.trim() && send(inputText.trim())}
          placeholder="Type a message…"
          disabled={isProcessing}
          style={{ flex: 1, padding: '11px 16px', borderRadius: '22px', border: '1px solid #2e2e42', background: '#21212e', color: '#eee', fontSize: '15px', outline: 'none' }}
        />
        <button
          onClick={() => inputText.trim() && send(inputText.trim())}
          disabled={!inputText.trim() || isProcessing}
          style={{ width: '44px', height: '44px', borderRadius: '50%', border: 'none', background: '#3a5f99', color: '#fff', fontSize: '20px', cursor: 'pointer', flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', opacity: inputText.trim() && !isProcessing ? 1 : 0.35 }}
        >↑</button>
      </div>

      {/* Mic zone */}
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '14px 0 20px', background: '#16161f', flexShrink: 0 }}>
        <button
          onMouseDown={startListening}
          onMouseUp={stopListening}
          onTouchStart={startListening}
          onTouchEnd={stopListening}
          disabled={isProcessing}
          style={{
            width: '70px', height: '70px', borderRadius: '50%', border: 'none', fontSize: '30px', cursor: 'pointer',
            transition: 'all 0.15s ease', WebkitTapHighlightColor: 'transparent', userSelect: 'none',
            ...(isListening
              ? { background: 'linear-gradient(145deg,#b02020,#d03030)', boxShadow: '0 4px 30px rgba(180,30,30,0.7)', transform: 'scale(0.94)' }
              : { background: 'linear-gradient(145deg,#3a5f99,#5a3faa)', boxShadow: '0 4px 20px rgba(58,95,153,0.45)' }),
          } as React.CSSProperties}
        >
          {isListening ? '🔴' : '🎙️'}
        </button>
        <div style={{ marginTop: '7px', fontSize: '12px', color: '#555' }}>
          {isListening ? 'Release to send' : 'Hold to speak'}
        </div>
      </div>
    </div>
  );
}
