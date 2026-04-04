'use client';

import { SidebarTrigger } from './ui/sidebar.js';

export function ChatHeader({ chatId }) {
  return (
    <header className="sticky top-0 flex items-center gap-2 bg-background px-2 py-1.5 md:px-2 z-10">
      {/* Mobile-only: open sidebar sheet */}
      <div className="md:hidden">
        <SidebarTrigger />
      </div>
    </header>
  );
}
