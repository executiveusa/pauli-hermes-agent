// App.tsx — Root layout with sidebar + panel routing

import React, { useEffect, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { Chat } from "./components/Chat";
import { VoiceInput } from "./components/VoiceInput";
import { MediaPanel } from "./components/MediaPanel";
import { AgentStatus } from "./components/AgentStatus";
import { SettingsPanel } from "./components/SettingsPanel";
import { useSettingsStore } from "./stores/settings";
import { useChatStore } from "./stores/chat";
import type { Panel } from "./lib/types";

export default function App() {
  const [activePanel, setActivePanel] = useState<Panel>("chat");

  const fetchBranding           = useSettingsStore((s) => s.fetchBranding);
  const refreshServicesStatus   = useSettingsStore((s) => s.refreshServicesStatus);
  const branding                = useSettingsStore((s) => s.branding);
  const newConversation         = useChatStore((s) => s.newConversation);
  const conversations           = useChatStore((s) => s.conversations);
  const setActiveConversation   = useChatStore((s) => s.setActiveConversation);

  // Bootstrap on mount
  useEffect(() => {
    fetchBranding();
    refreshServicesStatus();

    // Start polling service status every 15 seconds
    const interval = setInterval(() => {
      refreshServicesStatus();
    }, 15_000);

    return () => clearInterval(interval);
  }, []);

  // Auto-select first conversation on load
  useEffect(() => {
    if (conversations.length > 0 && !useChatStore.getState().activeConversationId) {
      setActiveConversation(conversations[0].id);
    }
  }, [conversations]);

  // Apply branding accent color to CSS variable
  useEffect(() => {
    if (branding?.colors.accent) {
      document.documentElement.style.setProperty("--accent", branding.colors.accent);
    }
    if (branding?.app_name) {
      document.title = branding.app_name;
    }
  }, [branding]);

  // Keyboard shortcut: N = new conversation (when on chat panel)
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if (e.key === "n" && activePanel === "chat") {
        newConversation();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [activePanel, newConversation]);

  return (
    <div className="app-layout">
      {/* Sidebar */}
      <Sidebar active={activePanel} onSelect={setActivePanel} />

      {/* Main */}
      <div className="main-content">
        {/* Titlebar drag region */}
        <div className="titlebar">
          <span className="titlebar__title">
            {branding?.window_title ?? "Hermes"}
          </span>
        </div>

        {/* Panel */}
        <div className="panel-container">
          {activePanel === "chat"     && <Chat />}
          {activePanel === "voice"    && <VoiceInput />}
          {activePanel === "media"    && <MediaPanel />}
          {activePanel === "agent"    && <AgentStatus />}
          {activePanel === "settings" && <SettingsPanel />}
        </div>
      </div>

      <style>{`
        .panel-container {
          flex: 1;
          overflow: hidden;
          display: flex;
          flex-direction: column;
        }
      `}</style>
    </div>
  );
}
