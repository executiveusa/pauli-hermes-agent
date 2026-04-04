// stores/settings.ts — Persistent settings store

import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { AllServicesStatus, Branding } from "../lib/types";
import * as bridge from "../lib/tauri-bridge";

interface SettingsState {
  branding:         Branding | null;
  servicesStatus:   AllServicesStatus | null;
  apiKeysSet:       Record<string, boolean>;
  localModelsCount: number;
  modelsDir:        string;

  fetchBranding:        () => Promise<void>;
  refreshServicesStatus: () => Promise<void>;
  checkApiKey:          (service: string) => Promise<void>;
  saveApiKey:           (service: string, key: string) => Promise<void>;
  refreshModelsInfo:    () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>()(
  immer((set) => ({
    branding:         null,
    servicesStatus:   null,
    apiKeysSet:       {},
    localModelsCount: 0,
    modelsDir:        "",

    fetchBranding: async () => {
      try {
        const branding = await bridge.getBranding();
        set((s) => { s.branding = branding; });
      } catch {
        // Use defaults
      }
    },

    refreshServicesStatus: async () => {
      try {
        const status = await bridge.getAllServicesStatus();
        set((s) => { s.servicesStatus = status; });
      } catch {
        // Silently ignore connectivity errors
      }
    },

    checkApiKey: async (service) => {
      try {
        const key = await bridge.getApiKey(service, false);
        set((s) => { s.apiKeysSet[service] = !!key; });
      } catch {
        set((s) => { s.apiKeysSet[service] = false; });
      }
    },

    saveApiKey: async (service, key) => {
      await bridge.saveApiKey(service, key);
      set((s) => { s.apiKeysSet[service] = true; });
    },

    refreshModelsInfo: async () => {
      try {
        const info = await bridge.scanModelsDir();
        set((s) => {
          s.localModelsCount = info.modelCount;
          s.modelsDir = info.modelsDir;
        });
      } catch {
        // fine
      }
    },
  }))
);
