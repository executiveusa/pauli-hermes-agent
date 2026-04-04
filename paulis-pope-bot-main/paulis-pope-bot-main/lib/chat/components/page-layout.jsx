'use client';

import { AppSidebar } from './app-sidebar.js';
import { SidebarProvider, SidebarInset } from './ui/sidebar.js';
import { ChatNavProvider } from './chat-nav-context.js';

function defaultNavigateToChat(id) {
  if (id) {
    window.location.href = `/chat/${id}`;
  } else {
    window.location.href = '/';
  }
}

export function PageLayout({ session, children }) {
  return (
    <ChatNavProvider value={{ activeChatId: null, navigateToChat: defaultNavigateToChat }}>
      <SidebarProvider>
        <AppSidebar user={session.user} />
        <SidebarInset>
          <div className="flex flex-col h-full max-w-4xl mx-auto w-full px-4 py-6">
            {children}
          </div>
        </SidebarInset>
      </SidebarProvider>
    </ChatNavProvider>
  );
}
