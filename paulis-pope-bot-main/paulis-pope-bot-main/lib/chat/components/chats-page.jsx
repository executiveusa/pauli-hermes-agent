'use client';

import { useState, useEffect, useRef } from 'react';
import { PageLayout } from './page-layout.js';
import { MessageIcon, TrashIcon, SearchIcon, PlusIcon, MoreHorizontalIcon, StarIcon, StarFilledIcon, PencilIcon } from './icons.js';
import { getChats, deleteChat, renameChat, starChat } from '../actions.js';
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator } from './ui/dropdown-menu.js';
import { ConfirmDialog } from './ui/confirm-dialog.js';
import { cn } from '../utils.js';

function groupChatsByDate(chats) {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today.getTime() - 86400000);
  const last7Days = new Date(today.getTime() - 7 * 86400000);
  const last30Days = new Date(today.getTime() - 30 * 86400000);

  const groups = {
    Starred: [],
    Today: [],
    Yesterday: [],
    'Last 7 Days': [],
    'Last 30 Days': [],
    Older: [],
  };

  for (const chat of chats) {
    if (chat.starred) {
      groups.Starred.push(chat);
      continue;
    }
    const date = new Date(chat.updatedAt);
    if (date >= today) {
      groups.Today.push(chat);
    } else if (date >= yesterday) {
      groups.Yesterday.push(chat);
    } else if (date >= last7Days) {
      groups['Last 7 Days'].push(chat);
    } else if (date >= last30Days) {
      groups['Last 30 Days'].push(chat);
    } else {
      groups.Older.push(chat);
    }
  }

  return groups;
}

function timeAgo(date) {
  const seconds = Math.floor((Date.now() - new Date(date).getTime()) / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  const months = Math.floor(days / 30);
  return `${months}mo ago`;
}

export function ChatsPage({ session }) {
  const [chats, setChats] = useState([]);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState('');

  const navigateToChat = (id) => {
    window.location.href = id ? `/chat/${id}` : '/';
  };

  const loadChats = async () => {
    try {
      const result = await getChats();
      setChats(result);
    } catch (err) {
      console.error('Failed to load chats:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadChats();
  }, []);

  useEffect(() => {
    const handler = () => loadChats();
    window.addEventListener('chatsupdated', handler);
    return () => window.removeEventListener('chatsupdated', handler);
  }, []);

  const handleDelete = async (chatId) => {
    setChats((prev) => prev.filter((c) => c.id !== chatId));
    const { success } = await deleteChat(chatId);
    if (!success) loadChats();
  };

  const handleStar = async (chatId) => {
    setChats((prev) =>
      prev.map((c) => (c.id === chatId ? { ...c, starred: c.starred ? 0 : 1 } : c))
    );
    const { success } = await starChat(chatId);
    if (!success) loadChats();
  };

  const handleRename = async (chatId, title) => {
    setChats((prev) =>
      prev.map((c) => (c.id === chatId ? { ...c, title } : c))
    );
    const { success } = await renameChat(chatId, title);
    if (!success) loadChats();
  };

  const filtered = query
    ? chats.filter((c) => c.title?.toLowerCase().includes(query.toLowerCase()))
    : chats;

  const grouped = groupChatsByDate(filtered);

  return (
    <PageLayout session={session}>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-semibold">Chats</h1>
        <button
          onClick={() => navigateToChat(null)}
          className="inline-flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium bg-foreground text-background hover:bg-foreground/90"
        >
          <PlusIcon size={14} />
          New chat
        </button>
      </div>

      {/* Search */}
      <div className="relative mb-4">
        <input
          type="text"
          placeholder="Search your chats..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="w-full rounded-md border border-input bg-background px-9 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
        />
        <div className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none">
          <SearchIcon size={16} />
        </div>
      </div>

      {/* Count */}
      <p className="text-sm text-muted-foreground mb-4">
        {filtered.length} {filtered.length === 1 ? 'chat' : 'chats'}
      </p>

      {/* Chat list */}
      {loading ? (
        <div className="flex flex-col gap-3">
          {[...Array(5)].map((_, i) => (
            <div key={i} className="h-14 animate-pulse rounded-md bg-border/50" />
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">
          {query ? 'No chats match your search.' : 'No chats yet. Start a conversation!'}
        </p>
      ) : (
        <div className="flex flex-col">
          {Object.entries(grouped).map(([label, groupChats]) =>
            groupChats.length > 0 ? (
              <div key={label} className="mb-4">
                <h2 className="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2">
                  {label}
                </h2>
                <div className="flex flex-col divide-y divide-border">
                  {groupChats.map((chat) => (
                    <ChatRow
                      key={chat.id}
                      chat={chat}
                      onNavigate={navigateToChat}
                      onDelete={handleDelete}
                      onStar={handleStar}
                      onRename={handleRename}
                    />
                  ))}
                </div>
              </div>
            ) : null
          )}
        </div>
      )}
    </PageLayout>
  );
}

function ChatRow({ chat, onNavigate, onDelete, onStar, onRename }) {
  const [hovered, setHovered] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [editing, setEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(chat.title || '');
  const inputRef = useRef(null);

  const showMenu = hovered || dropdownOpen;

  useEffect(() => {
    if (editing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editing]);

  const startRename = () => {
    setEditTitle(chat.title || '');
    setEditing(true);
  };

  const saveRename = () => {
    const trimmed = editTitle.trim();
    if (trimmed && trimmed !== chat.title) {
      onRename(chat.id, trimmed);
    }
    setEditing(false);
  };

  const cancelRename = () => {
    setEditing(false);
    setEditTitle(chat.title || '');
  };

  return (
    <div
      className="relative group flex items-center gap-3 px-3 py-3 cursor-pointer hover:bg-muted/50 rounded-md"
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={() => !editing && onNavigate(chat.id)}
    >
      <MessageIcon size={16} />
      <div className="flex-1 min-w-0">
        {editing ? (
          <input
            ref={inputRef}
            type="text"
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') saveRename();
              if (e.key === 'Escape') cancelRename();
            }}
            onBlur={saveRename}
            onClick={(e) => e.stopPropagation()}
            className="w-full text-sm bg-background border border-input rounded px-1.5 py-0.5 focus:outline-none focus:ring-2 focus:ring-ring"
          />
        ) : (
          <span
            className="text-sm truncate block"
            onDoubleClick={(e) => {
              e.stopPropagation();
              startRename();
            }}
          >
            {chat.title || 'New Chat'}
          </span>
        )}
        <span className="text-xs text-muted-foreground">
          Last message {timeAgo(chat.updatedAt)}
        </span>
      </div>
      {!editing && (
        <div className={cn(
          'shrink-0',
          showMenu ? 'opacity-100' : 'opacity-0 pointer-events-none'
        )}>
          <DropdownMenu open={dropdownOpen} onOpenChange={setDropdownOpen}>
            <DropdownMenuTrigger>
              <button
                className={cn(
                  'rounded-md p-1.5',
                  'text-muted-foreground hover:text-foreground hover:bg-muted'
                )}
                aria-label="Chat options"
              >
                <MoreHorizontalIcon size={14} />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" side="bottom">
              <DropdownMenuItem
                onClick={(e) => {
                  e.stopPropagation();
                  onStar(chat.id);
                }}
              >
                {chat.starred ? <StarFilledIcon size={14} /> : <StarIcon size={14} />}
                {chat.starred ? 'Unstar' : 'Star'}
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={(e) => {
                  e.stopPropagation();
                  startRename();
                }}
              >
                <PencilIcon size={14} />
                Rename
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="text-destructive hover:text-destructive"
                onClick={(e) => {
                  e.stopPropagation();
                  setConfirmDelete(true);
                }}
              >
                <TrashIcon size={14} />
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      )}
      <ConfirmDialog
        open={confirmDelete}
        title="Delete chat?"
        description="This will permanently delete this chat and all its messages."
        confirmLabel="Delete"
        onConfirm={() => {
          setConfirmDelete(false);
          onDelete(chat.id);
        }}
        onCancel={() => setConfirmDelete(false)}
      />
    </div>
  );
}
