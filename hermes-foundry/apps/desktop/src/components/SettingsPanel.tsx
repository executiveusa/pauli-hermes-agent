// components/SettingsPanel.tsx — API keys, model config, branding preview

import React, { useEffect, useState } from "react";
import { useSettingsStore } from "../stores/settings";
import { useChatStore } from "../stores/chat";
import * as bridge from "../lib/tauri-bridge";
import type { LocalModel } from "../lib/types";

const SERVICES = [
  { key: "anthropic",  label: "Anthropic",  hint: "sk-ant-…",  url: "https://console.anthropic.com" },
  { key: "openai",     label: "OpenAI",     hint: "sk-…",      url: "https://platform.openai.com/api-keys" },
  { key: "groq",       label: "Groq",       hint: "gsk_…",     url: "https://console.groq.com" },
  { key: "openrouter", label: "OpenRouter", hint: "sk-or-…",   url: "https://openrouter.ai/keys" },
];

export function SettingsPanel() {
  const branding         = useSettingsStore((s) => s.branding);
  const fetchBranding    = useSettingsStore((s) => s.fetchBranding);
  const checkApiKey      = useSettingsStore((s) => s.checkApiKey);
  const saveApiKey       = useSettingsStore((s) => s.saveApiKey);
  const apiKeysSet       = useSettingsStore((s) => s.apiKeysSet);
  const refreshModels    = useSettingsStore((s) => s.refreshModelsInfo);
  const localModels      = useSettingsStore((s) => s.localModelsCount);

  const currentModel     = useChatStore((s) => s.currentModel);
  const setModel         = useChatStore((s) => s.setModel);
  const useLocalModel    = useChatStore((s) => s.useLocalModel);
  const setUseLocalModel = useChatStore((s) => s.setUseLocalModel);
  const temperature      = useChatStore((s) => s.temperature);
  const setTemperature   = useChatStore((s) => s.setTemperature);

  const [llmModels, setLlmModels]     = useState<LocalModel[]>([]);
  const [keyInputs, setKeyInputs]     = useState<Record<string, string>>({});
  const [saving, setSaving]           = useState<Record<string, boolean>>({});
  const [saved, setSaved]             = useState<Record<string, boolean>>({});

  useEffect(() => {
    fetchBranding();
    refreshModels();
    SERVICES.forEach((s) => checkApiKey(s.key));
    bridge.listLocalModels().then(setLlmModels).catch(() => {});
  }, []);

  const handleSaveKey = async (service: string) => {
    const key = keyInputs[service]?.trim();
    if (!key) return;
    setSaving((p) => ({ ...p, [service]: true }));
    try {
      await saveApiKey(service, key);
      setKeyInputs((p) => ({ ...p, [service]: "" }));
      setSaved((p) => ({ ...p, [service]: true }));
      setTimeout(() => setSaved((p) => ({ ...p, [service]: false })), 2000);
    } finally {
      setSaving((p) => ({ ...p, [service]: false }));
    }
  };

  return (
    <div className="settings-panel">
      <div className="settings-panel__header">
        <h2 className="settings-panel__title">Settings</h2>
      </div>

      {/* Model */}
      <section className="settings-panel__section">
        <h3 className="settings-panel__section-title">Model</h3>

        <div className="settings-panel__field">
          <div className="settings-panel__field-header">
            <label className="settings-panel__label">Use local LLM</label>
            <span className="text-secondary text-xs">Requires downloaded model + llama-server running</span>
          </div>
          <label className="toggle">
            <input
              type="checkbox"
              checked={useLocalModel}
              onChange={(e) => setUseLocalModel(e.target.checked)}
            />
            <span className="toggle__track" />
            <span className="toggle__label">{useLocalModel ? "Local" : "Remote API"}</span>
          </label>
        </div>

        {useLocalModel ? (
          <div className="settings-panel__field">
            <label className="settings-panel__label">Local model ({localModels} installed)</label>
            {llmModels.length > 0 ? (
              <select
                className="input"
                value={currentModel}
                onChange={(e) => setModel(e.target.value)}
              >
                <option value="default">Auto (first available)</option>
                {llmModels.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.name}{m.quantization ? ` [${m.quantization}]` : ""}{m.sizeGb ? ` · ${m.sizeGb.toFixed(1)} GB` : ""}
                  </option>
                ))}
              </select>
            ) : (
              <p className="text-secondary text-sm">
                No models found in ~/.hermes/models/<br />
                Run <code>scripts/install-models.sh</code> to download.
              </p>
            )}
          </div>
        ) : (
          <div className="settings-panel__field">
            <label className="settings-panel__label">Remote model</label>
            <input
              className="input"
              value={currentModel === "default" ? "" : currentModel}
              onChange={(e) => setModel(e.target.value || "default")}
              placeholder="anthropic/claude-opus-4-5 or leave empty for default"
            />
          </div>
        )}

        <div className="settings-panel__field">
          <label className="settings-panel__label">Temperature: {temperature.toFixed(2)}</label>
          <input
            type="range"
            min={0}
            max={2}
            step={0.05}
            value={temperature}
            onChange={(e) => setTemperature(parseFloat(e.target.value))}
            className="slider"
          />
          <div className="slider-hints">
            <span className="text-xs text-secondary">Deterministic</span>
            <span className="text-xs text-secondary">Creative</span>
          </div>
        </div>
      </section>

      {/* API Keys */}
      <section className="settings-panel__section">
        <h3 className="settings-panel__section-title">API keys</h3>
        <p className="text-secondary text-sm">
          Keys are stored in the OS system keyring, never on disk as plaintext.
        </p>
        <div className="api-keys-list">
          {SERVICES.map((svc) => (
            <div key={svc.key} className="api-key-row">
              <div className="api-key-row__header">
                <span className="api-key-row__service">{svc.label}</span>
                {apiKeysSet[svc.key] && (
                  <span className="badge badge-success">Set</span>
                )}
              </div>
              <div className="api-key-row__input">
                <input
                  type="password"
                  className="input flex-1"
                  placeholder={svc.hint}
                  value={keyInputs[svc.key] ?? ""}
                  onChange={(e) => setKeyInputs((p) => ({ ...p, [svc.key]: e.target.value }))}
                  onKeyDown={(e) => e.key === "Enter" && handleSaveKey(svc.key)}
                  autoComplete="off"
                  spellCheck={false}
                />
                <button
                  className="btn btn-primary"
                  onClick={() => handleSaveKey(svc.key)}
                  disabled={saving[svc.key] || !keyInputs[svc.key]?.trim()}
                >
                  {saving[svc.key] ? "…" : saved[svc.key] ? "✓ Saved" : "Save"}
                </button>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Branding info */}
      {branding && (
        <section className="settings-panel__section">
          <h3 className="settings-panel__section-title">Branding</h3>
          <div className="branding-preview">
            <div className="info-row">
              <span className="info-row__label">App name</span>
              <span className="info-row__value">{branding.app_name}</span>
            </div>
            <div className="info-row">
              <span className="info-row__label">Company</span>
              <span className="info-row__value">{branding.company_name}</span>
            </div>
            <div className="info-row">
              <span className="info-row__label">Accent</span>
              <span className="info-row__value flex items-center gap-2">
                <span
                  style={{
                    display: "inline-block",
                    width: 12,
                    height: 12,
                    borderRadius: 2,
                    background: branding.colors.accent,
                    flexShrink: 0,
                  }}
                />
                {branding.colors.accent}
              </span>
            </div>
            <div className="info-row">
              <span className="info-row__label">Support</span>
              <span className="info-row__value">
                <a href="#" onClick={(e) => e.preventDefault()}>{branding.support_url}</a>
              </span>
            </div>
          </div>
          <p className="text-secondary text-xs mt-2">
            Customize branding in <code>branding.json</code> at the workspace root.
          </p>
        </section>
      )}

      <style>{settingsStyles}</style>
    </div>
  );
}

const settingsStyles = `
  .settings-panel {
    display: flex;
    flex-direction: column;
    gap: var(--sp-6);
    padding: var(--sp-6);
    max-width: 600px;
    margin: 0 auto;
    width: 100%;
    overflow-y: auto;
    height: 100%;
  }

  .settings-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .settings-panel__title {
    font-size: var(--text-xl);
    font-weight: var(--weight-semi);
  }

  .settings-panel__section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    padding-bottom: var(--sp-6);
    border-bottom: 1px solid var(--border);
  }

  .settings-panel__section:last-child {
    border-bottom: none;
  }

  .settings-panel__section-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .settings-panel__field {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .settings-panel__field-header {
    display: flex;
    align-items: baseline;
    gap: var(--sp-3);
  }

  .settings-panel__label {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  /* Toggle */
  .toggle {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    cursor: pointer;
  }

  .toggle input { display: none; }

  .toggle__track {
    width: 36px;
    height: 20px;
    background: var(--surface-3);
    border: 1px solid var(--border-2);
    border-radius: 10px;
    position: relative;
    transition: background var(--transition);
    flex-shrink: 0;
  }

  .toggle__track::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    background: var(--text-tertiary);
    border-radius: 50%;
    transition: transform var(--transition), background var(--transition);
  }

  .toggle input:checked + .toggle__track {
    background: var(--accent-dim);
    border-color: var(--accent);
  }

  .toggle input:checked + .toggle__track::after {
    transform: translateX(16px);
    background: var(--accent);
  }

  .toggle__label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  /* Slider */
  .slider {
    width: 100%;
    accent-color: var(--accent);
    cursor: pointer;
    height: 4px;
  }

  .slider-hints {
    display: flex;
    justify-content: space-between;
  }

  /* API keys */
  .api-keys-list {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .api-key-row {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--sp-3) var(--sp-4);
  }

  .api-key-row__header {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .api-key-row__service {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
  }

  .api-key-row__input {
    display: flex;
    gap: var(--sp-2);
  }

  /* Branding */
  .branding-preview {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--sp-4);
  }

  .info-row {
    display: flex;
    gap: var(--sp-4);
    padding: 5px 0;
    border-bottom: 1px solid var(--border);
    font-size: var(--text-sm);
  }

  .info-row:last-child { border-bottom: none; }

  .info-row__label {
    width: 80px;
    flex-shrink: 0;
    color: var(--text-secondary);
  }

  .info-row__value {
    flex: 1;
    color: var(--text-primary);
  }
`;
