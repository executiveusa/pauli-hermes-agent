'use client';

import Link from 'next/link';
import { useStore } from '@/lib/store';
import FreeModeToggle from '@/components/FreeModeToggle';

const CARDS = [
  { href: '/chat', icon: '💬', title: 'Chat', desc: 'Text conversation with Hermes', color: '#3a5f99' },
  { href: '/voice', icon: '🎙️', title: 'Voice', desc: 'Hold-to-talk voice interface', color: '#5a3faa' },
  { href: '/sandbox', icon: '🏰', title: 'Sandbox', desc: 'Safe agent execution & monitoring', color: '#0d7377' },
  { href: '/skills', icon: '⚙️', title: 'Skills', desc: 'Enable/disable agent capabilities', color: '#0d7377' },
  { href: '/models', icon: '🧠', title: 'Models', desc: 'Choose inference provider', color: '#7a3f60' },
];

const PROVIDERS = [
  { key: 'groq', label: '⚡ Groq / OpenAI', desc: 'Fast, primary provider' },
  { key: 'nvidia', label: '🚀 NVIDIA NIM', desc: 'Free tier — kimi-k2-thinking' },
  { key: 'mercury', label: '💎 Mercury', desc: 'Premium reasoning' },
];

export default function Dashboard() {
  const { freeMode, messages, enabledSkills } = useStore();

  return (
    <div style={{ padding: '24px 16px', maxWidth: '720px', margin: '0 auto', width: '100%' }}>
      <div style={{ marginBottom: '28px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, margin: 0, color: '#eee' }}>Control Room</h1>
        <p style={{ margin: '4px 0 0', color: '#555', fontSize: '13px' }}>Hermes AI Agent Dashboard</p>
      </div>

      {/* FREE MODE banner */}
      <div style={{
        background: freeMode ? 'rgba(34,197,94,0.08)' : 'rgba(58,95,153,0.08)',
        border: `1px solid ${freeMode ? '#22c55e33' : '#3a5f9933'}`,
        borderRadius: '12px',
        padding: '16px 20px',
        marginBottom: '24px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        gap: '12px',
        flexWrap: 'wrap',
      }}>
        <div>
          <div style={{ fontSize: '14px', fontWeight: 600, color: freeMode ? '#22c55e' : '#7aa0cc' }}>
            {freeMode ? '⚡ FREE MODE — NVIDIA NIM' : '💠 Premium Mode — Paid Provider'}
          </div>
          <div style={{ fontSize: '12px', color: '#555', marginTop: '2px' }}>
            {freeMode
              ? 'All inference → NVIDIA NIM (moonshotai/kimi-k2-thinking) — $0'
              : 'Routes through Groq → Mercury → NVIDIA based on availability'}
          </div>
        </div>
        <FreeModeToggle />
      </div>

      {/* Quick nav cards */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px', marginBottom: '28px' }}>
        {CARDS.map(({ href, icon, title, desc, color }) => (
          <Link key={href} href={href} style={{ textDecoration: 'none' }}>
            <div
              style={{ background: '#16161f', border: '1px solid #252535', borderRadius: '12px', padding: '18px', cursor: 'pointer', transition: 'border-color 0.15s ease' }}
              onMouseEnter={(e) => ((e.currentTarget as HTMLDivElement).style.borderColor = color)}
              onMouseLeave={(e) => ((e.currentTarget as HTMLDivElement).style.borderColor = '#252535')}
            >
              <div style={{ fontSize: '24px', marginBottom: '8px' }}>{icon}</div>
              <div style={{ fontSize: '14px', fontWeight: 600, color: '#eee' }}>{title}</div>
              <div style={{ fontSize: '12px', color: '#555', marginTop: '3px' }}>{desc}</div>
            </div>
          </Link>
        ))}
      </div>

      {/* Stats */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px', marginBottom: '28px' }}>
        {[
          { label: 'Messages', value: messages.length },
          { label: 'Active Skills', value: enabledSkills.length },
          { label: 'Mode', value: freeMode ? 'Free ⚡' : 'Premium' },
        ].map(({ label, value }) => (
          <div key={label} style={{ background: '#16161f', border: '1px solid #252535', borderRadius: '10px', padding: '14px 16px', textAlign: 'center' }}>
            <div style={{ fontSize: '20px', fontWeight: 700, color: '#ccc' }}>{value}</div>
            <div style={{ fontSize: '11px', color: '#555', marginTop: '2px' }}>{label}</div>
          </div>
        ))}
      </div>

      {/* Providers */}
      <div style={{ background: '#16161f', border: '1px solid #252535', borderRadius: '12px', padding: '18px' }}>
        <div style={{ fontSize: '11px', fontWeight: 600, color: '#444', letterSpacing: '0.08em', marginBottom: '14px', textTransform: 'uppercase' }}>
          Provider Chain
        </div>
        {PROVIDERS.map(({ key, label, desc }, i) => (
          <div key={key} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 0', borderBottom: i < PROVIDERS.length - 1 ? '1px solid #1a1a28' : 'none' }}>
            <div>
              <div style={{ fontSize: '13px', color: '#ccc' }}>{label}</div>
              <div style={{ fontSize: '11px', color: '#444', marginTop: '1px' }}>{desc}</div>
            </div>
            <div style={{ width: '7px', height: '7px', borderRadius: '50%', background: (key === 'nvidia' && freeMode) ? '#22c55e' : (key === 'groq' && !freeMode) ? '#22c55e' : '#2a2a3a' }} />
          </div>
        ))}
      </div>
    </div>
  );
}
