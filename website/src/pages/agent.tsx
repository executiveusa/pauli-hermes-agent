import React, { useState, useRef, useEffect } from 'react';
import Layout from '@theme/Layout';

export default function VoiceAgent(): JSX.Element {
  const [isListening, setIsListening] = useState(false);
  const [transcript, setTranscript] = useState('');
  const [response, setResponse] = useState('');
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [useMercury, setUseMercury] = useState(true); // Mercury toggle
  const [useNvidia, setUseNvidia] = useState(true); // NVIDIA NIM toggle
  const recognitionRef = useRef<any>(null);
  const synthesisRef = useRef<any>(null);

  // For production, connect to your VPS; for local dev, use localhost
  const API_BASE =
    typeof window !== 'undefined' && window.location.hostname !== 'localhost'
      ? `https://${window.location.hostname}:8642`
      : 'http://localhost:8642';

  useEffect(() => {
    // Initialize Web Speech API
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (SpeechRecognition) {
      recognitionRef.current = new SpeechRecognition();
      recognitionRef.current.continuous = false;
      recognitionRef.current.interimResults = true;
      recognitionRef.current.lang = 'en-US';

      recognitionRef.current.onstart = () => setIsListening(true);
      recognitionRef.current.onend = () => setIsListening(false);

      recognitionRef.current.onresult = (event: any) => {
        let interim = '';
        for (let i = event.resultIndex; i < event.results.length; i++) {
          const transcriptPart = event.results[i][0].transcript;
          if (event.results[i].isFinal) {
            setTranscript((prev) => prev + transcriptPart + ' ');
          } else {
            interim += transcriptPart;
          }
        }
      };

      recognitionRef.current.onerror = (event: any) => {
        console.error('Speech recognition error', event.error);
      };
    }
  }, []);

  const startListening = () => {
    if (recognitionRef.current) {
      setTranscript('');
      recognitionRef.current.start();
    }
  };

  const stopListening = () => {
    if (recognitionRef.current) {
      recognitionRef.current.stop();
    }
  };

  const speakResponse = async (text: string) => {
    setIsSpeaking(true);
    try {
      // Use browser's native text-to-speech API
      const utterance = new SpeechSynthesisUtterance(text);
      utterance.rate = 0.95;
      utterance.pitch = 1;
      utterance.volume = 1;

      utterance.onend = () => setIsSpeaking(false);
      utterance.onerror = () => setIsSpeaking(false);

      window.speechSynthesis.cancel(); // Cancel any ongoing speech
      window.speechSynthesis.speak(utterance);
    } catch (error) {
      console.error('TTS error:', error);
      setIsSpeaking(false);
    }
  };

  const sendMessage = async () => {
    if (!transcript.trim()) return;

    setIsProcessing(true);
    setError(null);
    try {
      const res = await fetch(`${API_BASE}/api/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: transcript,
          agent_type: 'hermes',
          providers: {
            mercury: useMercury,
            nvidia: useNvidia,
          },
        }),
      });

      if (!res.ok) {
        const errorData = await res.json().catch(() => ({ detail: res.statusText }));
        throw new Error(errorData.detail || `API error: ${res.status}`);
      }

      const data = await res.json();
      const agentResponse = data.response || data.message || 'Done';
      setResponse(agentResponse);
      await speakResponse(agentResponse);
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      console.error('API error:', errorMsg);
      setError(`❌ ${errorMsg}`);
      setResponse(`Error: ${errorMsg}`);
      await speakResponse(`Error: ${errorMsg}`);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <Layout
      title="Hermes Voice Agent"
      description="Talk to your AI agent — no typing required"
    >
      <div style={styles.container}>
        <div style={styles.card}>
          <h1 style={styles.title}>🎤 Hermes Voice Agent</h1>
          <p style={styles.subtitle}>Speak naturally. Agent listens and acts.</p>

          <div style={styles.visualizer}>
            <div
              style={{
                ...styles.waveform,
                opacity: isListening ? 1 : 0.3,
              }}
            >
              🌊
            </div>
            <div style={styles.status}>
              {isListening
                ? '👂 Listening...'
                : isProcessing
                ? '⚙️ Processing...'
                : isSpeaking
                ? '🔊 Speaking...'
                : '💬 Ready'}
            </div>
          </div>

          {error && (
            <div style={styles.errorBox}>
              {error}
            </div>
          )}

          <div style={styles.toggleContainer}>
            <label style={styles.toggleLabel}>
              <input
                type="checkbox"
                checked={useNvidia}
                onChange={(e) => setUseNvidia(e.target.checked)}
                style={styles.toggleInput}
              />
              <span style={styles.toggleText}>🚀 NVIDIA NIM (Free)</span>
            </label>
            <label style={styles.toggleLabel}>
              <input
                type="checkbox"
                checked={useMercury}
                onChange={(e) => setUseMercury(e.target.checked)}
                style={styles.toggleInput}
              />
              <span style={styles.toggleText}>💎 Mercury Inception Labs</span>
            </label>
          </div>

          <div style={styles.transcript}>
            <strong>You:</strong>
            <p style={styles.transcriptText}>
              {transcript || '(waiting for speech...)'}
            </p>
          </div>

          <div style={styles.response}>
            <strong>Agent:</strong>
            <p style={styles.responseText}>
              {response || '(agent response will appear here...)'}
            </p>
          </div>

          <div style={styles.buttonGroup}>
            <button
              onMouseDown={startListening}
              onMouseUp={stopListening}
              onTouchStart={startListening}
              onTouchEnd={stopListening}
              style={{
                ...styles.button,
                ...styles.micButton,
                ...(isListening && styles.activeButton),
              }}
            >
              🎙️ Hold to Speak
            </button>

            <button
              onClick={sendMessage}
              disabled={!transcript.trim() || isProcessing}
              style={{
                ...styles.button,
                ...styles.sendButton,
                opacity: !transcript.trim() || isProcessing ? 0.5 : 1,
              }}
            >
              {isProcessing ? '⏳ Sending...' : '✈️ Send'}
            </button>

            <button
              onClick={() => {
                setTranscript('');
                setResponse('');
              }}
              style={styles.button}
            >
              🔄 Clear
            </button>
          </div>

          <div style={styles.info}>
            <small>
              💡 Tip: Hold the microphone button, speak your command, and release. The agent will respond with voice.
            </small>
          </div>
        </div>
      </div>
    </Layout>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    minHeight: '100vh',
    padding: '20px',
    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
  },
  card: {
    background: 'white',
    borderRadius: '20px',
    padding: '40px',
    maxWidth: '500px',
    width: '100%',
    boxShadow: '0 20px 60px rgba(0,0,0,0.3)',
  },
  title: {
    textAlign: 'center',
    fontSize: '32px',
    margin: '0 0 10px 0',
    color: '#333',
  },
  subtitle: {
    textAlign: 'center',
    color: '#666',
    marginBottom: '30px',
    fontSize: '16px',
  },
  visualizer: {
    textAlign: 'center',
    marginBottom: '30px',
  },
  waveform: {
    fontSize: '48px',
    marginBottom: '10px',
    transition: 'opacity 0.2s',
  },
  status: {
    fontSize: '18px',
    fontWeight: 'bold',
    color: '#667eea',
    minHeight: '24px',
  },
  transcript: {
    background: '#f5f5f5',
    padding: '15px',
    borderRadius: '10px',
    marginBottom: '15px',
    minHeight: '60px',
  },
  transcriptText: {
    margin: '5px 0 0 0',
    color: '#333',
    fontSize: '14px',
    fontStyle: 'italic',
  },
  response: {
    background: '#e8f5e9',
    padding: '15px',
    borderRadius: '10px',
    marginBottom: '20px',
    minHeight: '60px',
  },
  responseText: {
    margin: '5px 0 0 0',
    color: '#333',
    fontSize: '14px',
  },
  buttonGroup: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '10px',
    marginBottom: '20px',
  },
  button: {
    padding: '12px 20px',
    fontSize: '16px',
    border: 'none',
    borderRadius: '10px',
    cursor: 'pointer',
    fontWeight: 'bold',
    transition: 'all 0.2s',
    background: '#667eea',
    color: 'white',
  },
  micButton: {
    gridColumn: '1 / -1',
    padding: '20px',
    fontSize: '18px',
    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
  },
  sendButton: {
    background: '#4CAF50',
  },
  activeButton: {
    background: '#ff5252',
    transform: 'scale(0.95)',
  },
  info: {
    textAlign: 'center',
    color: '#999',
    fontSize: '13px',
  },
  errorBox: {
    background: '#ffebee',
    border: '2px solid #f44336',
    borderRadius: '8px',
    padding: '12px',
    marginBottom: '15px',
    color: '#c62828',
    fontSize: '14px',
    fontWeight: 'bold',
  },
  toggleContainer: {
    display: 'flex',
    flexDirection: 'column',
    gap: '10px',
    marginBottom: '20px',
    padding: '12px',
    background: '#f9f9f9',
    borderRadius: '10px',
    border: '1px solid #e0e0e0',
  },
  toggleLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    cursor: 'pointer',
    userSelect: 'none',
  },
  toggleInput: {
    width: '18px',
    height: '18px',
    cursor: 'pointer',
  },
  toggleText: {
    fontSize: '14px',
    fontWeight: '500',
    color: '#333',
  },
};
