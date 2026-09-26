'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import FreeModeToggle from './FreeModeToggle';

const LINKS = [
  { href: '/', label: 'Dashboard', icon: '⬡' },
  { href: '/chat', label: 'Chat', icon: '💬' },
  { href: '/voice', label: 'Voice', icon: '🎙️' },
  { href: '/skills', label: 'Skills', icon: '⚙️' },
  { href: '/models', label: 'Models', icon: '🧠' },
  { href: '/settings', label: 'Settings', icon: '🔧' },
];

export default function Nav() {
  const path = usePathname();

  return (
    <nav style={{
      display: 'flex',
      alignItems: 'center',
      gap: '4px',
      padding: '0 16px',
      height: '52px',
      background: '#16161f',
      borderBottom: '1px solid #252535',
      position: 'sticky',
      top: 0,
      zIndex: 100,
      flexShrink: 0,
    }}>
      <span style={{ fontSize: '15px', fontWeight: 700, color: '#eee', marginRight: '12px', letterSpacing: '-0.02em' }}>
        Hermes
      </span>

      <div style={{ display: 'flex', gap: '2px', flex: 1, overflowX: 'auto' }}>
        {LINKS.map(({ href, label, icon }) => {
          const active = path === href;
          return (
            <Link
              key={href}
              href={href}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '5px',
                padding: '5px 10px',
                borderRadius: '8px',
                fontSize: '13px',
                fontWeight: active ? 600 : 400,
                color: active ? '#eee' : '#666',
                background: active ? '#252535' : 'transparent',
                textDecoration: 'none',
                whiteSpace: 'nowrap',
                transition: 'all 0.15s ease',
              }}
            >
              <span style={{ fontSize: '13px' }}>{icon}</span>
              <span className="hidden-mobile">{label}</span>
            </Link>
          );
        })}
      </div>

      <FreeModeToggle compact />
    </nav>
  );
}
