import React, { useState, useRef, useEffect, useCallback } from 'react';
import Layout from '@theme/Layout';

interface Message {
  id: string;
  role: 'user' | 'agent';
  text: string;
  provider?: string;
}

export default function VoiceAgent(): React.ReactElement {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isListening, setIsListening] = useState(false);
  const [transcript, setTranscript] = useState('');
  const [response, setResponse] = useState('');
  const [provider, setProvider] = useState('');
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [inputText, setInputText] = useState('');
  const [interimText, setInterimText] = useState('');
  const recognitionRef = useRef<any>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const pendingTextRef = useRef('');

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, interimText]);

  useEffect(() => {
    const Ctor = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!Ctor) return;
    const rec = new Ctor();
    rec.continuous = false;
    rec.interimResults = true;
    rec.lang = 'en-US';

    rec.onstart = () => setIsListening(true);

    rec.onresult = (event: any) => {
      let final = '';
      let interim = '';
      for (let i = event.resultIndex; i < event.results.length; i++) {
        if (event.results[i].isFinal) final += event.results[i][0].transcript;
        else interim += event.results[i][0].transcript;
      }
      if (final) pendingTextRef.current += final + ' ';
      setInterimText(interim);
    };

    rec.onend = () => {
      setIsListening(false);
      setInterimText('');
      const text = pendingTextRef.current.trim();
      pendingTextRef.current = '';
      if (text) sendMessage(text);
    };

    recognitionRef.current = rec;
  }, []);

  const speakText = (text: string) => {
    setIsSpeaking(true);
    const clean = text.replace(/^[💎🚀⚡]\s*/, '');
    const utt = new SpeechSynthesisUtterance(clean);
    utt.rate = 1.0;
    utt.onend = () => setIsSpeaking(false);
    utt.onerror = () => setIsSpeaking(false);
    window.speechSynthesis.cancel();
    window.speechSynthesis.speak(utt);
  };

  const sendMessage = useCallback(async (text: string) => {
    if (!text || isProcessing) return;
    setInputText('');

    setMessages(prev => [...prev, { id: `u-${Date.now()}`, role: 'user', text }]);
    setIsProcessing(true);

    try {
      const res = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: text,
          agent_type: 'hermes',
          providers: { mercury: true, nvidia: true },
        }),
      });
      const data = await res.json();
      const agentResponse = data.response || data.message || 'Done';
      const provider = data.provider || '';
      setMessages(prev => [...prev, { id: `a-${Date.now()}`, role: 'agent', text: agentResponse, provider }]);
      speakText(agentResponse);
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      console.error('API error:', errorMsg);
      setMessages(prev => [...prev, { id: `a-${Date.now()}`, role: 'agent', text: `Error: ${errorMsg}` }]);
      speakText(`Error: ${errorMsg}`);
    } finally {
      setIsProcessing(false);
    }
  }, [isProcessing]);

  const startListening = (e: React.MouseEvent | React.TouchEvent) => {
    e.preventDefault();
    if (recognitionRef.current && !isListening && !isProcessing) {
      pendingTextRef.current = '';
      recognitionRef.current.start();
    }
  };

  const stopListening = (e: React.MouseEvent | React.TouchEvent) => {
    e.preventDefault();
    recognitionRef.current?.stop();
  };

  const handleSend = () => {
    const text = inputText.trim();
    if (text) sendMessage(text);
  };

  const providerLabel = (p?: string) => {
    if (!p) return null;
    const map: Record<string, string> = { synthia: '⚡ Groq/OpenAI', mercury: '💎 Mercury', 'nvidia-nim': '🚀 NVIDIA' };
    return map[p] || p;
  };

  return (
    <Layout title="Hermes" description="AI voice agent">
      <div style={S.root}>
        <div style={S.topBar}>
          <span style={S.topTitle}>Hermes Agent</span>
          {messages.length > 0 && (
            <button style={S.clearBtn} onClick={() => { setMessages([]); window.speechSynthesis.cancel(); }}>
              Clear
            </button>
          )}
        </div>

        <div style={S.feed}>
          {messages.length === 0 && (
            <div style={S.empty}>Hold the mic and speak a command</div>
          )}
          {messages.map(m => (
            <div key={m.id} style={{ ...S.row, ...(m.role === 'user' ? S.rowUser : {}) }}>
              <div style={{ ...S.bubble, ...(m.role === 'user' ? S.bubbleUser : S.bubbleAgent) }}>
                <span>{m.text}</span>
                {m.provider && <span style={S.badge}>{providerLabel(m.provider)}</span>}
              </div>
            </div>
          ))}
          {isListening && interimText && (
            <div style={{ ...S.row, ...S.rowUser }}>
              <div style={{ ...S.bubble, ...S.bubbleUser, opacity: 0.55 }}>{interimText}</div>
            </div>
          )}
          {isProcessing && (
            <div style={S.row}>
              <div style={{ ...S.bubble, ...S.bubbleAgent, color: '#888' }}>…</div>
            </div>
          )}
          <div ref={messagesEndRef} />
        </div>

        <div style={S.micZone}>
          <button
            onMouseDown={startListening}
            onMouseUp={stopListening}
            onTouchStart={startListening}
            onTouchEnd={stopListening}
            style={{ ...S.mic, ...(isListening ? S.micOn : {}) }}
            disabled={isProcessing}
          >
            {isListening ? '🔴' : '🎙️'}
          </button>
          <div style={S.hint}>{isListening ? 'Release to send' : 'Hold to speak'}</div>
        </div>
      </div>
    </Layout>
  );
}

const S: Record<string, React.CSSProperties> = {
  root: {
    display: 'flex',
    flexDirection: 'column',
    height: 'calc(100dvh - 60px)',
    background: '#0d0d14',
    color: '#eee',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    overflow: 'hidden',
  },
  topBar: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '10px 16px',
    background: '#16161f',
    borderBottom: '1px solid #252535',
    flexShrink: 0,
  },
  topTitle: { fontSize: '15px', fontWeight: 600, color: '#ccc' },
  clearBtn: {
    background: 'none',
    border: '1px solid #333',
    color: '#888',
    borderRadius: '6px',
    padding: '4px 10px',
    fontSize: '12px',
    cursor: 'pointer',
  },
  feed: {
    flex: 1,
    overflowY: 'auto',
    padding: '16px 12px 8px',
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    WebkitOverflowScrolling: 'touch',
  } as React.CSSProperties,
  empty: {
    textAlign: 'center',
    color: '#44445a',
    fontSize: '14px',
    marginTop: '60px',
  },
  row: { display: 'flex', justifyContent: 'flex-start' },
  rowUser: { justifyContent: 'flex-end' },
  bubble: {
    maxWidth: '78%',
    padding: '10px 14px',
    borderRadius: '18px',
    fontSize: '15px',
    lineHeight: 1.45,
    display: 'flex',
    flexDirection: 'column',
    gap: '5px',
    wordBreak: 'break-word',
  },
  bubbleUser: {
    background: '#3a5f99',
    color: '#fff',
    borderBottomRightRadius: '5px',
  },
  bubbleAgent: {
    background: '#1c1c2a',
    color: '#dde',
    border: '1px solid #2a2a40',
    borderBottomLeftRadius: '5px',
  },
  responseHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '4px',
  },
  providerBadge: {
    fontSize: '11px',
    fontWeight: '600',
    color: '#fff',
    background: '#4CAF50',
    borderRadius: '12px',
    padding: '2px 8px',
    letterSpacing: '0.3px',
  },
  badge: { fontSize: '10px', color: '#666', alignSelf: 'flex-end' },
  bar: {
    display: 'flex',
    gap: '8px',
    padding: '8px 12px',
    background: '#16161f',
    borderTop: '1px solid #252535',
    flexShrink: 0,
  },
  input: {
    flex: 1,
    padding: '11px 16px',
    borderRadius: '22px',
    border: '1px solid #2e2e42',
    background: '#21212e',
    color: '#eee',
    fontSize: '15px',
    outline: 'none',
  },
  sendBtn: {
    width: '44px',
    height: '44px',
    borderRadius: '50%',
    border: 'none',
    background: '#3a5f99',
    color: '#fff',
    fontSize: '20px',
    cursor: 'pointer',
    flexShrink: 0,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  micZone: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    padding: '14px 0 20px',
    background: '#16161f',
    flexShrink: 0,
  },
  mic: {
    width: '70px',
    height: '70px',
    borderRadius: '50%',
    border: 'none',
    background: 'linear-gradient(145deg, #3a5f99, #5a3faa)',
    fontSize: '30px',
    cursor: 'pointer',
    boxShadow: '0 4px 20px rgba(58,95,153,0.45)',
    transition: 'all 0.15s ease',
    WebkitTapHighlightColor: 'transparent',
    userSelect: 'none',
  } as React.CSSProperties,
  micOn: {
    background: 'linear-gradient(145deg, #b02020, #d03030)',
    boxShadow: '0 4px 30px rgba(180,30,30,0.7)',
    transform: 'scale(0.94)',
  },
  hint: { marginTop: '7px', fontSize: '12px', color: '#555' },
};
