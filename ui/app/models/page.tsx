'use client';

import { useStore, Provider } from '@/lib/store';

const MODELS: { id: Provider; icon: string; name: string; desc: string; cost: string; speed: string; free: boolean }[] = [
  {
    id: 'auto',
    icon: '🔄',
    name: 'Auto (Recommended)',
    desc: 'Groq → Mercury → NVIDIA — tries providers in order of availability',
    cost: 'Varies',
    speed: 'Fast',
    free: false,
  },
  {
    id: 'groq',
    icon: '⚡',
    name: 'Groq — Llama 3.3 70B',
    desc: 'Blazing fast free inference via Groq API. llama-3.3-70b-versatile',
    cost: '$0',
    speed: 'Very Fast',
    free: true,
  },
  {
    id: 'nvidia',
    icon: '🚀',
    name: 'NVIDIA NIM — Kimi K2',
    desc: 'Free NVIDIA developer tier. moonshotai/kimi-k2-thinking',
    cost: '$0',
    speed: 'Fast',
    free: true,
  },
  {
    id: 'mercury',
    icon: '💎',
    name: 'Mercury Inception Labs',
    desc: 'High-quality diffusion language model. Best for complex reasoning',
    cost: 'Pay-as-go',
    speed: 'Fast',
    free: false,
  },
];

export default function ModelsPage() {
  const { provider, setProvider, freeMode, setFreeMode } = useStore();

  return (
    <div style={{ padding: '24px 16px', maxWidth: '720px', margin: '0 auto', width: '100%' }}>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, margin: 0, color: '#eee' }}>Models</h1>
        <p style={{ margin: '4px 0 0', color: '#555', fontSize: '13px' }}>
          Choose your inference provider
        </p>
      </div>

      {/* FREE MODE callout */}
      <div
        onClick={() => setFreeMode(!freeMode)}
        style={{
          background: freeMode ? 'rgba(34,197,94,0.1)' : 'rgba(30,30,50,0.5)',
          border: `1px solid ${freeMode ? '#22c55e55' : '#252535'}`,
          borderRadius: '12px',
          padding: '16px 20px',
          marginBottom: '24px',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: '12px',
        }}
      >
        <div>
          <div style={{ fontSize: '14px', fontWeight: 700, color: freeMode ? '#22c55e' : '#888' }}>
            ⚡ FREE MODE
          </div>
          <div style={{ fontSize: '12px', color: '#555', marginTop: '2px' }}>
            Force all inference through NVIDIA NIM — 100% free, 40 req/min limit
          </div>
        </div>
        <div style={{
          width: '44px', height: '24px', borderRadius: '12px', flexShrink: 0,
          background: freeMode ? '#22c55e' : '#2a2a3a',
          position: 'relative', transition: 'background 0.2s ease',
        }}>
          <div style={{
            position: 'absolute', top: '4px', left: freeMode ? '23px' : '4px',
            width: '16px', height: '16px', borderRadius: '50%', background: '#fff',
            transition: 'left 0.2s ease', boxShadow: '0 1px 3px rgba(0,0,0,0.3)',
          }} />
        </div>
      </div>

      {/* Model cards */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
        {MODELS.map((m) => {
          const selected = provider === m.id;
          return (
            <div
              key={m.id}
              onClick={() => setProvider(m.id)}
              style={{
                padding: '16px',
                background: '#16161f',
                border: `1px solid ${selected ? '#3a5f99' : '#252535'}`,
                borderRadius: '12px',
                cursor: 'pointer',
                transition: 'border-color 0.15s ease',
                display: 'flex',
                gap: '14px',
                alignItems: 'flex-start',
              }}
            >
              <span style={{ fontSize: '24px', flexShrink: 0, marginTop: '1px' }}>{m.icon}</span>
              <div style={{ flex: 1 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                  <span style={{ fontSize: '14px', fontWeight: 600, color: selected ? '#eee' : '#999' }}>{m.name}</span>
                  {m.free && <span style={{ fontSize: '10px', fontWeight: 700, color: '#22c55e', background: 'rgba(34,197,94,0.12)', padding: '1px 6px', borderRadius: '4px' }}>FREE</span>}
                </div>
                <div style={{ fontSize: '12px', color: '#444', marginBottom: '8px' }}>{m.desc}</div>
                <div style={{ display: 'flex', gap: '16px' }}>
                  <span style={{ fontSize: '11px', color: '#555' }}>Cost: {m.cost}</span>
                  <span style={{ fontSize: '11px', color: '#555' }}>Speed: {m.speed}</span>
                </div>
              </div>
              <div style={{
                width: '18px', height: '18px', borderRadius: '50%', flexShrink: 0,
                border: `2px solid ${selected ? '#3a5f99' : '#333'}`,
                background: selected ? '#3a5f99' : 'transparent',
                display: 'flex', alignItems: 'center', justifyContent: 'center', marginTop: '2px',
              }}>
                {selected && <div style={{ width: '6px', height: '6px', borderRadius: '50%', background: '#fff' }} />}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
