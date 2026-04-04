// stores/chat.ts — Zustand store for conversation state

import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { Message, Conversation } from "../lib/types";
import * as bridge from "../lib/tauri-bridge";

const STORAGE_KEY = "hermes:conversations";

interface ChatState {
  conversations:       Conversation[];
  activeConversationId: string | null;
  messages:            Record<string, Message[]>;
  streaming:           boolean;
  currentModel:        string;
  useLocalModel:       boolean;
  temperature:         number;

  // Actions
  setActiveConversation: (id: string | null) => void;
  newConversation:       () => string;
  sendMessage:           (content: string) => Promise<void>;
  stopStreaming:          () => void;
  clearConversation:     (id: string) => void;
  deleteConversation:    (id: string) => void;
  setModel:              (model: string) => void;
  setUseLocalModel:      (value: boolean) => void;
  setTemperature:        (value: number) => void;
  appendToken:           (conversationId: string, token: string) => void;
  finalizeStream:        (conversationId: string) => void;
  addErrorMessage:       (conversationId: string, error: string) => void;
}

let currentUnlisten: (() => void) | null = null;

export const useChatStore = create<ChatState>()(
  immer((set, get) => ({
    conversations:        loadConversations(),
    activeConversationId: null,
    messages:             {},
    streaming:            false,
    currentModel:         "default",
    useLocalModel:        true,
    temperature:          0.7,

    setActiveConversation: (id) => {
      set((s) => { s.activeConversationId = id; });
    },

    newConversation: () => {
      const id = crypto.randomUUID();
      const conversation: Conversation = {
        id,
        title:        "New conversation",
        model:        get().currentModel,
        messageCount: 0,
        createdAt:    Date.now(),
        updatedAt:    Date.now(),
      };
      set((s) => {
        s.conversations.unshift(conversation);
        s.messages[id] = [];
        s.activeConversationId = id;
      });
      saveConversations(get().conversations);
      return id;
    },

    sendMessage: async (content) => {
      const state = get();
      let convId = state.activeConversationId;
      if (!convId) {
        convId = get().newConversation();
      }

      const userMsg: Message = {
        id:             crypto.randomUUID(),
        role:           "user",
        content,
        conversationId: convId,
        timestamp:      Date.now(),
      };

      const assistantMsg: Message = {
        id:             crypto.randomUUID(),
        role:           "assistant",
        content:        "",
        conversationId: convId,
        timestamp:      Date.now(),
        streaming:      true,
      };

      set((s) => {
        if (!s.messages[convId!]) s.messages[convId!] = [];
        s.messages[convId!].push(userMsg, assistantMsg);
        s.streaming = true;
        // Update conversation title if first message
        const conv = s.conversations.find((c) => c.id === convId);
        if (conv) {
          if (conv.messageCount === 0) {
            conv.title = content.slice(0, 60);
          }
          conv.messageCount += 2;
          conv.updatedAt = Date.now();
          conv.model = s.currentModel;
        }
      });

      // Clean up previous listeners
      if (currentUnlisten) {
        currentUnlisten();
        currentUnlisten = null;
      }

      try {
        const { unlisten } = await bridge.streamMessage(
          {
            message:        content,
            conversationId: convId,
            model:          state.currentModel !== "default" ? state.currentModel : undefined,
            useLocal:       state.useLocalModel,
            temperature:    state.temperature,
          },
          {
            onToken: (cid, token) => get().appendToken(cid, token),
            onDone:  (cid) => {
              get().finalizeStream(cid);
              if (currentUnlisten) { currentUnlisten(); currentUnlisten = null; }
            },
            onError: (cid, error) => {
              get().addErrorMessage(cid, error);
              if (currentUnlisten) { currentUnlisten(); currentUnlisten = null; }
            },
          }
        );
        currentUnlisten = unlisten;
      } catch (err) {
        get().addErrorMessage(convId, String(err));
        set((s) => { s.streaming = false; });
      }
    },

    stopStreaming: () => {
      if (currentUnlisten) { currentUnlisten(); currentUnlisten = null; }
      bridge.stopGeneration().catch(() => {});
      const convId = get().activeConversationId;
      if (convId) get().finalizeStream(convId);
    },

    clearConversation: (id) => {
      set((s) => { s.messages[id] = []; });
    },

    deleteConversation: (id) => {
      set((s) => {
        s.conversations = s.conversations.filter((c) => c.id !== id);
        delete s.messages[id];
        if (s.activeConversationId === id) {
          s.activeConversationId = s.conversations[0]?.id ?? null;
        }
      });
      saveConversations(get().conversations);
    },

    setModel: (model) => set((s) => { s.currentModel = model; }),
    setUseLocalModel: (v) => set((s) => { s.useLocalModel = v; }),
    setTemperature: (v) => set((s) => { s.temperature = v; }),

    appendToken: (conversationId, token) => {
      set((s) => {
        const msgs = s.messages[conversationId];
        if (!msgs) return;
        const last = msgs[msgs.length - 1];
        if (last && last.role === "assistant" && last.streaming) {
          last.content += token;
        }
      });
    },

    finalizeStream: (conversationId) => {
      set((s) => {
        const msgs = s.messages[conversationId];
        if (!msgs) return;
        const last = msgs[msgs.length - 1];
        if (last && last.streaming) {
          last.streaming = false;
        }
        s.streaming = false;
      });
    },

    addErrorMessage: (conversationId, error) => {
      set((s) => {
        const msgs = s.messages[conversationId];
        if (!msgs) return;
        const last = msgs[msgs.length - 1];
        if (last && last.role === "assistant") {
          last.streaming = false;
          last.error = error;
          if (!last.content) {
            last.content = `Error: ${error}`;
          }
        }
        s.streaming = false;
      });
    },
  }))
);

function loadConversations(): Conversation[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveConversations(conversations: Conversation[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(conversations));
  } catch {
    // quota exceeded — silently ignore
  }
}
