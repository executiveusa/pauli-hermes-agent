'use client';

import { useStore } from '@/lib/store';

const ALL_SKILLS = [
  { id: 'memory', icon: '🧠', name: 'Memory', desc: 'Remember facts about people and events' },
  { id: 'contacts', icon: '👤', name: 'Contacts / Rolodex', desc: 'Track and recall contact information' },
  { id: 'calendar', icon: '📅', name: 'Calendar', desc: 'Schedule and remind about events' },
  { id: 'web-search', icon: '🔍', name: 'Web Search', desc: 'Search the web for current info' },
  { id: 'email', icon: '📧', name: 'Email', desc: 'Draft and send email messages' },
  { id: 'code', icon: '💻', name: 'Code Assistant', desc: 'Write, review, and debug code' },
  { id: 'image', icon: '🖼️', name: 'Image Gen', desc: 'Generate images via DALL-E / Stable Diffusion' },
  { id: 'voice-notes', icon: '🎙️', name: 'Voice Notes', desc: 'Create and replay voice memos' },
  { id: 'telegram', icon: '✈️', name: 'Telegram Bot', desc: 'Send/receive messages via Telegram' },
  { id: 'weather', icon: '🌤️', name: 'Weather', desc: 'Get current weather and forecasts' },
  { id: 'calculator', icon: '🔢', name: 'Calculator', desc: 'Perform complex calculations' },
  { id: 'translate', icon: '🌐', name: 'Translate', desc: 'Translate text between languages' },
];

export default function SkillsPage() {
  const { enabledSkills, toggleSkill } = useStore();

  return (
    <div style={{ padding: '24px 16px', maxWidth: '720px', margin: '0 auto', width: '100%' }}>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, margin: 0, color: '#eee' }}>Skills</h1>
        <p style={{ margin: '4px 0 0', color: '#555', fontSize: '13px' }}>
          {enabledSkills.length} of {ALL_SKILLS.length} skills enabled
        </p>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        {ALL_SKILLS.map(({ id, icon, name, desc }) => {
          const enabled = enabledSkills.includes(id);
          return (
            <div
              key={id}
              onClick={() => toggleSkill(id)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '14px',
                padding: '14px 16px',
                background: '#16161f',
                border: `1px solid ${enabled ? '#3a5f9966' : '#252535'}`,
                borderRadius: '12px',
                cursor: 'pointer',
                transition: 'border-color 0.15s ease',
              }}
            >
              <span style={{ fontSize: '22px', flexShrink: 0 }}>{icon}</span>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: '14px', fontWeight: 600, color: enabled ? '#eee' : '#777' }}>{name}</div>
                <div style={{ fontSize: '12px', color: '#444', marginTop: '2px' }}>{desc}</div>
              </div>
              {/* Toggle switch */}
              <div style={{
                width: '40px', height: '22px', borderRadius: '11px', flexShrink: 0,
                background: enabled ? '#3a5f99' : '#2a2a3a',
                position: 'relative',
                transition: 'background 0.2s ease',
              }}>
                <div style={{
                  position: 'absolute',
                  top: '3px',
                  left: enabled ? '21px' : '3px',
                  width: '16px',
                  height: '16px',
                  borderRadius: '50%',
                  background: '#fff',
                  transition: 'left 0.2s ease',
                  boxShadow: '0 1px 3px rgba(0,0,0,0.3)',
                }} />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
