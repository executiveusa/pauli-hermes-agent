'use client';

import { useState, useEffect } from 'react';
import { PageLayout } from './page-layout.js';
import { ClockIcon, ZapIcon, KeyIcon } from './icons.js';

const TABS = [
  { id: 'crons', label: 'Crons', href: '/settings/crons', icon: ClockIcon },
  { id: 'triggers', label: 'Triggers', href: '/settings/triggers', icon: ZapIcon },
  { id: 'secrets', label: 'Secrets', href: '/settings/secrets', icon: KeyIcon },
];

export function SettingsLayout({ session, children }) {
  const [activePath, setActivePath] = useState('');

  useEffect(() => {
    setActivePath(window.location.pathname);
  }, []);

  return (
    <PageLayout session={session}>
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-semibold">Settings</h1>
      </div>

      {/* Tab navigation */}
      <div className="flex gap-1 border-b border-border mb-6">
        {TABS.map((tab) => {
          const isActive = activePath === tab.href || activePath.startsWith(tab.href + '/');
          const Icon = tab.icon;
          return (
            <a
              key={tab.id}
              href={tab.href}
              className={`inline-flex items-center gap-2 px-3 py-2 text-sm font-medium border-b-2 transition-colors ${
                isActive
                  ? 'border-foreground text-foreground'
                  : 'border-transparent text-muted-foreground hover:text-foreground hover:border-border'
              }`}
            >
              <Icon size={14} />
              {tab.label}
            </a>
          );
        })}
      </div>

      {/* Tab content */}
      {children}
    </PageLayout>
  );
}
