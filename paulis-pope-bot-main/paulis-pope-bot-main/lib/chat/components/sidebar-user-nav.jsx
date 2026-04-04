'use client';

import { signOut } from 'next-auth/react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './ui/dropdown-menu.js';
import { SidebarMenu, SidebarMenuItem, SidebarMenuButton } from './ui/sidebar.js';
import { SettingsIcon, SunIcon, MoonIcon, BugIcon, LogOutIcon } from './icons.js';
import { cn } from '../utils.js';

export function SidebarUserNav({ user, collapsed }) {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => setMounted(true), []);

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton className={cn(collapsed ? 'justify-center' : 'justify-between')}>
              <div className={cn('flex items-center overflow-hidden', collapsed ? '' : 'gap-2')}>
                <div className="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary text-primary-foreground text-xs font-medium">
                  {(user?.email?.[0] || 'U').toUpperCase()}
                </div>
                {!collapsed && (
                  <span className="truncate text-sm">{user?.email || 'User'}</span>
                )}
              </div>
              {!collapsed && (
                <svg className="size-4 text-muted-foreground" width={16} height={16} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
                  <path d="m7 15 5 5 5-5" />
                  <path d="m7 9 5-5 5 5" />
                </svg>
              )}
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" side="top" className="w-56">
            <DropdownMenuItem onClick={() => { window.location.href = '/settings'; }}>
              <SettingsIcon size={14} />
              <span className="ml-2">Settings</span>
            </DropdownMenuItem>
            {mounted && (
              <DropdownMenuItem onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}>
                {theme === 'dark' ? <SunIcon size={14} /> : <MoonIcon size={14} />}
                <span className="ml-2">{theme === 'dark' ? 'Light Mode' : 'Dark Mode'}</span>
              </DropdownMenuItem>
            )}
            <DropdownMenuItem onClick={() => window.open('https://github.com/stephengpope/thepopebot/issues', '_blank')}>
              <BugIcon size={14} />
              <span className="ml-2">Report Issues</span>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onClick={() => signOut({ callbackUrl: '/' })}
              className="text-destructive"
            >
              <LogOutIcon size={14} />
              <span className="ml-2">Sign Out</span>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
