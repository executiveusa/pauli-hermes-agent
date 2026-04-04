// components/Sidebar.tsx — Navigation sidebar

import React from "react";
import type { Panel } from "../lib/types";
import { useChatStore } from "../stores/chat";
import { useSettingsStore } from "../stores/settings";

interface SidebarProps {
  active:   Panel;
  onSelect: (panel: Panel) => void;
}

const NAV_ITEMS: { id: Panel; icon: string; label: string }[] = [
  { id: "chat",     icon: "󰭹", label: "Chat" },
  { id: "voice",    icon: "󰥐", label: "Voice" },
  { id: "media",    icon: "󰈸", label: "Media" },
  { id: "agent",    icon: "󱙺", label: "Agent" },
  { id: "settings", icon: "󰒓", label: "Settings" },
];

export function Sidebar({ active, onSelect }: SidebarProps) {
  const conversations      = useChatStore((s) => s.conversations);
  const activeConvId       = useChatStore((s) => s.activeConversationId);
  const setActiveConv      = useChatStore((s) => s.setActiveConversation);
  const newConversation    = useChatStore((s) => s.newConversation);
  const deleteConversation = useChatStore((s) => s.deleteConversation);
  const servicesStatus     = useSettingsStore((s) => s.servicesStatus);
  const branding           = useSettingsStore((s) => s.branding);

  const apiOnline = servicesStatus?.hermes_api.healthy ?? false;
  const llmOnline = servicesStatus?.llama_server.healthy ?? false;

  return (
    <aside className="sidebar">
      {/* App Name */}
      <div className="sidebar__header">
        <span className="sidebar__appname">
          {branding?.app_name ?? "Hermes"}
        </span>
        <div className="sidebar__status-row">
          <span
            className={`status-dot ${apiOnline ? "online" : "offline"}`}
            title={`API ${apiOnline ? "online" : "offline"}`}
          />
          <span
            className={`status-dot ${llmOnline ? "online" : "offline"}`}
            title={`LLM ${llmOnline ? "online" : "offline"}`}
          />
        </div>
      </div>

      {/* Nav */}
      <nav className="sidebar__nav">
        {NAV_ITEMS.map((item) => (
          <button
            key={item.id}
            className={`sidebar__nav-item ${active === item.id ? "active" : ""}`}
            onClick={() => onSelect(item.id)}
          >
            <span className="sidebar__nav-icon">{item.icon}</span>
            <span className="sidebar__nav-label">{item.label}</span>
          </button>
        ))}
      </nav>

      {/* Chat history (only when on chat panel) */}
      {active === "chat" && (
        <div className="sidebar__history">
          <div className="sidebar__history-header">
            <span>Conversations</span>
            <button
              className="btn-icon"
              title="New conversation"
              onClick={() => { newConversation(); }}
            >
              +
            </button>
          </div>
          <ul className="sidebar__history-list">
            {conversations.map((conv) => (
              <li
                key={conv.id}
                className={`sidebar__conv-item ${conv.id === activeConvId ? "active" : ""}`}
                onClick={() => { setActiveConv(conv.id); }}
              >
                <span className="truncate sidebar__conv-title">{conv.title}</span>
                <button
                  className="sidebar__conv-delete btn-icon"
                  title="Delete"
                  onClick={(e) => {
                    e.stopPropagation();
                    deleteConversation(conv.id);
                  }}
                >
                  ✕
                </button>
              </li>
            ))}
            {conversations.length === 0 && (
              <li className="sidebar__history-empty">No conversations yet</li>
            )}
          </ul>
        </div>
      )}

      <style>{`
        .sidebar {
          width: var(--sidebar-width);
          background: var(--surface);
          border-right: 1px solid var(--border);
          display: flex;
          flex-direction: column;
          flex-shrink: 0;
          overflow: hidden;
        }

        .sidebar__header {
          padding: var(--sp-4) var(--sp-3) var(--sp-3);
          border-bottom: 1px solid var(--border);
          display: flex;
          align-items: center;
          justify-content: space-between;
        }

        .sidebar__appname {
          font-size: var(--text-md);
          font-weight: var(--weight-semi);
          color: var(--text-primary);
          letter-spacing: -0.01em;
        }

        .sidebar__status-row {
          display: flex;
          gap: 4px;
          align-items: center;
        }

        .sidebar__nav {
          padding: var(--sp-2) 0;
          border-bottom: 1px solid var(--border);
        }

        .sidebar__nav-item {
          display: flex;
          align-items: center;
          gap: var(--sp-2);
          width: 100%;
          padding: 6px var(--sp-3);
          color: var(--text-secondary);
          font-size: var(--text-sm);
          text-align: left;
          border-radius: 0;
          transition: background var(--transition), color var(--transition);
        }

        .sidebar__nav-item:hover {
          background: var(--surface-2);
          color: var(--text-primary);
        }

        .sidebar__nav-item.active {
          background: var(--accent-dim);
          color: var(--accent);
        }

        .sidebar__nav-icon {
          font-size: 14px;
          width: 18px;
          text-align: center;
          flex-shrink: 0;
        }

        .sidebar__nav-label {
          font-weight: var(--weight-medium);
        }

        .sidebar__history {
          flex: 1;
          display: flex;
          flex-direction: column;
          overflow: hidden;
        }

        .sidebar__history-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: var(--sp-3) var(--sp-3) var(--sp-2);
          font-size: var(--text-xs);
          font-weight: var(--weight-medium);
          color: var(--text-tertiary);
          text-transform: uppercase;
          letter-spacing: 0.06em;
        }

        .sidebar__history-list {
          list-style: none;
          overflow-y: auto;
          flex: 1;
          padding: 0 var(--sp-1);
        }

        .sidebar__conv-item {
          display: flex;
          align-items: center;
          gap: var(--sp-2);
          padding: 5px var(--sp-2);
          border-radius: var(--radius);
          cursor: pointer;
          font-size: var(--text-sm);
          color: var(--text-secondary);
          transition: background var(--transition), color var(--transition);
        }

        .sidebar__conv-item:hover {
          background: var(--surface-2);
          color: var(--text-primary);
        }

        .sidebar__conv-item:hover .sidebar__conv-delete {
          opacity: 1;
        }

        .sidebar__conv-item.active {
          background: var(--surface-2);
          color: var(--text-primary);
        }

        .sidebar__conv-title {
          flex: 1;
          min-width: 0;
        }

        .sidebar__conv-delete {
          opacity: 0;
          font-size: 10px;
          color: var(--text-tertiary);
          transition: opacity var(--transition);
          padding: 2px 4px;
          flex-shrink: 0;
        }

        .sidebar__conv-delete:hover {
          color: var(--error);
          background: rgba(239,68,68,0.1);
        }

        .sidebar__history-empty {
          padding: var(--sp-4) var(--sp-3);
          font-size: var(--text-sm);
          color: var(--text-tertiary);
        }
      `}</style>
    </aside>
  );
}
