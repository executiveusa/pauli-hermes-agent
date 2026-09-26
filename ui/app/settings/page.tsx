'use client';

import { useState } from 'react';
import { useStore } from '@/lib/store';

export default function SettingsPage() {
  const { apiUrl, setApiUrl } = useStore();
  const [localUrl, setLocalUrl] = useState(apiUrl);
  const [saved, setSaved] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);

  const save = () => {
    setApiUrl(localUrl);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const testConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const res = await fetch('/api/health');
      const data = await res.json();
      setTestResult(`✅ API: ${JSON.stringify(data.api)}`);
    } catch (err) {
      setTestResult(`❌ ${err instanceof Error ? err.message : 'Connection failed'}`);
    } finally {
      setTesting(false);
    }
  };

  const CONFIG_ITEMS = [
    {
      label: 'VPS API URL',
      hint: 'Direct URL to your Hermes API server (only change if self-hosting)',
      value: localUrl,
      onChange: setLocalUrl,
      placeholder: 'http://31.220.58.212:8642',
    },
  ];

  return (
    <div style={{ padding: '24px 16px', maxWidth: '720px', margin: '0 auto', width: '100%' }}>
      <div style={{ marginBottom: '28px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, margin: 0, color: '#eee' }}>Settings</h1>
        <p style={{ margin: '4px 0 0', color: '#555', fontSize: '13px' }}>Configure Hermes agent connections</p>
      </div>

      {/* API Configuration */}
      <section style={{ background: '#16161f', border: '1px solid #252535', borderRadius: '12px', padding: '20px', marginBottom: '20px' }}>
        <div style={{ fontSize: '11px', fontWeight: 600, color: '#444', letterSpacing: '0.08em', textTransform: 'uppercase', marginBottom: '16px' }}>
          API Configuration
        </div>

        {CONFIG_ITEMS.map(({ label, hint, value, onChange, placeholder }) => (
          <div key={label} style={{ marginBottom: '16px' }}>
            <label style={{ display: 'block', fontSize: '13px', color: '#aaa', marginBottom: '6px' }}>{label}</label>
            <input
              value={value}
              onChange={(e) => onChange(e.target.value)}
              placeholder={placeholder}
              style={{ width: '100%', padding: '10px 14px', borderRadius: '8px', border: '1px solid #2e2e42', background: '#21212e', color: '#eee', fontSize: '14px', outline: 'none', boxSizing: 'border-box' } as React.CSSProperties}
            />
            <div style={{ fontSize: '11px', color: '#444', marginTop: '4px' }}>{hint}</div>
          </div>
        ))}

        <div style={{ display: 'flex', gap: '10px', alignItems: 'center', flexWrap: 'wrap' }}>
          <button
            onClick={save}
            style={{ padding: '9px 20px', borderRadius: '8px', border: 'none', background: '#3a5f99', color: '#fff', fontSize: '13px', fontWeight: 600, cursor: 'pointer' }}
          >
            {saved ? '✅ Saved' : 'Save'}
          </button>
          <button
            onClick={testConnection}
            disabled={testing}
            style={{ padding: '9px 20px', borderRadius: '8px', border: '1px solid #333', background: 'none', color: '#888', fontSize: '13px', cursor: 'pointer' }}
          >
            {testing ? 'Testing…' : 'Test Connection'}
          </button>
          {testResult && (
            <span style={{ fontSize: '12px', color: testResult.startsWith('✅') ? '#22c55e' : '#e55' }}>
              {testResult}
            </span>
          )}
        </div>
      </section>

      {/* Info section */}
      <section style={{ background: '#16161f', border: '1px solid #252535', borderRadius: '12px', padding: '20px', marginBottom: '20px' }}>
        <div style={{ fontSize: '11px', fontWeight: 600, color: '#444', letterSpacing: '0.08em', textTransform: 'uppercase', marginBottom: '16px' }}>
          Infrastructure
        </div>
        {[
          { label: 'VPS', value: '31.220.58.212 (Hostinger)' },
          { label: 'API Server', value: ':8642 — FastAPI' },
          { label: 'NIM Proxy', value: ':8082 — Free NVIDIA Inference' },
          { label: 'AionUI', value: ':3001 — Full Chat Interface' },
          { label: 'Vercel', value: 'pauli-hermes-agent.vercel.app' },
        ].map(({ label, value }) => (
          <div key={label} style={{ display: 'flex', justifyContent: 'space-between', padding: '7px 0', borderBottom: '1px solid #1a1a28', fontSize: '13px' }}>
            <span style={{ color: '#666' }}>{label}</span>
            <span style={{ color: '#aaa', fontFamily: 'monospace', fontSize: '12px' }}>{value}</span>
          </div>
        ))}
      </section>

      {/* About */}
      <section style={{ background: '#16161f', border: '1px solid #252535', borderRadius: '12px', padding: '20px' }}>
        <div style={{ fontSize: '11px', fontWeight: 600, color: '#444', letterSpacing: '0.08em', textTransform: 'uppercase', marginBottom: '12px' }}>
          About
        </div>
        <div style={{ fontSize: '13px', color: '#555', lineHeight: 1.6 }}>
          Hermes is a voice-first AI agent with memory, contact management, and multi-provider inference.
          Provider chain: <span style={{ color: '#aaa' }}>Synthia/Groq → Mercury → NVIDIA NIM</span>.
        </div>
        <div style={{ marginTop: '12px', fontSize: '12px', color: '#333' }}>
          <a href="https://github.com/executiveusa/pauli-hermes-agent" style={{ color: '#3a5f99', textDecoration: 'none' }}>
            GitHub →
          </a>
        </div>
      </section>
    </div>
  );
}
