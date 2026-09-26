import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface Message {
  id: string;
  role: 'user' | 'agent';
  text: string;
  provider?: string;
  ts: number;
}

export type Provider = 'auto' | 'groq' | 'nvidia' | 'mercury';

interface HermesState {
  // FREE MODE
  freeMode: boolean;
  setFreeMode: (v: boolean) => void;

  // Provider selection
  provider: Provider;
  setProvider: (p: Provider) => void;

  // Chat history
  messages: Message[];
  addMessage: (m: Message) => void;
  clearMessages: () => void;

  // API server URL (defaults to relative /api/chat for Vercel proxy)
  apiUrl: string;
  setApiUrl: (url: string) => void;

  // Skills
  enabledSkills: string[];
  toggleSkill: (id: string) => void;
}

export const useStore = create<HermesState>()(
  persist(
    (set, get) => ({
      freeMode: false,
      setFreeMode: (v) => set({ freeMode: v }),

      provider: 'auto',
      setProvider: (p) => set({ provider: p }),

      messages: [],
      addMessage: (m) => set((s) => ({ messages: [...s.messages, m] })),
      clearMessages: () => set({ messages: [] }),

      apiUrl: '/api/chat',
      setApiUrl: (url) => set({ apiUrl: url }),

      enabledSkills: ['memory', 'calendar', 'contacts', 'web-search'],
      toggleSkill: (id) =>
        set((s) => ({
          enabledSkills: s.enabledSkills.includes(id)
            ? s.enabledSkills.filter((x) => x !== id)
            : [...s.enabledSkills, id],
        })),
    }),
    {
      name: 'hermes-store',
      partialize: (s) => ({
        freeMode: s.freeMode,
        provider: s.provider,
        apiUrl: s.apiUrl,
        enabledSkills: s.enabledSkills,
      }),
    }
  )
);
