// components/VoiceInput.tsx — Whisper-powered voice transcription panel

import React, { useState, useRef, useCallback } from "react";
import * as bridge from "../lib/tauri-bridge";
import type { VoiceModel, TranscriptionResult } from "../lib/types";
import { useChatStore } from "../stores/chat";

type RecordingState = "idle" | "recording" | "processing";

export function VoiceInput() {
  const [state, setState]                 = useState<RecordingState>("idle");
  const [transcript, setTranscript]       = useState<TranscriptionResult | null>(null);
  const [error, setError]                 = useState<string | null>(null);
  const [models, setModels]               = useState<VoiceModel[]>([]);
  const [modelsLoaded, setModelsLoaded]   = useState(false);
  const [voiceReady, setVoiceReady]       = useState<boolean | null>(null);
  const [insertOnDone, setInsertOnDone]   = useState(true);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef        = useRef<Blob[]>([]);
  const sendMessage      = useChatStore((s) => s.sendMessage);

  const checkVoiceServer = useCallback(async () => {
    try {
      const status = await bridge.getVoiceServerStatus();
      setVoiceReady(status.ready);
    } catch {
      setVoiceReady(false);
    }
  }, []);

  const loadModels = useCallback(async () => {
    try {
      const list = await bridge.listVoiceModels();
      setModels(list);
      setModelsLoaded(true);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  React.useEffect(() => {
    checkVoiceServer();
    loadModels();
  }, [checkVoiceServer, loadModels]);

  const startRecording = useCallback(async () => {
    setError(null);
    setTranscript(null);
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const recorder = new MediaRecorder(stream, { mimeType: "audio/webm;codecs=opus" });
      chunksRef.current = [];

      recorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunksRef.current.push(e.data);
      };
      recorder.onstop = async () => {
        stream.getTracks().forEach((t) => t.stop());
        await processRecording();
      };

      mediaRecorderRef.current = recorder;
      recorder.start(250); // collect every 250ms
      setState("recording");
      await bridge.startRecording?.();
    } catch (err) {
      setError("Microphone access denied. Check browser permissions.");
      setState("idle");
    }
  }, []);

  const stopRecording = useCallback(() => {
    mediaRecorderRef.current?.stop();
    setState("processing");
  }, []);

  const processRecording = async () => {
    if (chunksRef.current.length === 0) {
      setState("idle");
      return;
    }

    try {
      const blob = new Blob(chunksRef.current, { type: "audio/webm" });

      // Write to a temp file so Rust can pick it up
      // In Tauri v2, we can write to AppLocalData
      const arrayBuffer = await blob.arrayBuffer();
      const uint8 = new Uint8Array(arrayBuffer);

      // Save to temp via Tauri FS
      const { writeFile, tempDir } = await import("@tauri-apps/plugin-fs");
      const tmpPath = `${await tempDir()}hermes_voice_${Date.now()}.webm`;
      await writeFile(tmpPath, uint8);

      // First convert to 16kHz WAV for whisper
      const { convertAudioForWhisper, transcribeFile } = await import("../lib/tauri-bridge");
      const converted = await convertAudioForWhisper(tmpPath);
      const result = await transcribeFile(converted.outputPath);

      setTranscript(result);

      if (insertOnDone && result.text.trim()) {
        await sendMessage(result.text.trim());
      }
    } catch (err) {
      setError(`Transcription failed: ${String(err)}`);
    } finally {
      setState("idle");
    }
  };

  return (
    <div className="voice-panel">
      <div className="voice-panel__header">
        <h2 className="voice-panel__title">Voice Input</h2>
        <div className="voice-panel__status">
          <span className={`status-dot ${voiceReady ? "online" : voiceReady === null ? "loading" : "offline"}`} />
          <span className="text-secondary text-sm">
            {voiceReady === null ? "Checking…" : voiceReady ? "Whisper ready" : "Whisper offline"}
          </span>
        </div>
      </div>

      {/* Record button */}
      <div className="voice-panel__record-section">
        <RecordButton
          recordingState={state}
          onStart={startRecording}
          onStop={stopRecording}
          disabled={voiceReady === false}
        />
        <p className="voice-panel__record-hint">
          {state === "idle"      && "Click to start recording"}
          {state === "recording" && "Recording… click to stop"}
          {state === "processing" && "Transcribing…"}
        </p>
      </div>

      {/* Transcript result */}
      {transcript && (
        <div className="voice-panel__result">
          <div className="voice-panel__result-header">
            <span className="badge badge-success">Transcribed</span>
            {transcript.language && (
              <span className="badge badge-neutral">{transcript.language}</span>
            )}
            {transcript.durationS && (
              <span className="badge badge-neutral">{transcript.durationS.toFixed(1)}s</span>
            )}
          </div>
          <p className="voice-panel__transcript selectable">{transcript.text}</p>
          {transcript.segments.length > 0 && (
            <details className="voice-panel__segments">
              <summary>Segments ({transcript.segments.length})</summary>
              {transcript.segments.map((seg) => (
                <div key={seg.id} className="voice-panel__segment">
                  <span className="voice-panel__segment-time mono">
                    {fmtTime(seg.start)} → {fmtTime(seg.end)}
                  </span>
                  <span className="selectable">{seg.text}</span>
                </div>
              ))}
            </details>
          )}
          {!insertOnDone && (
            <button
              className="btn btn-primary mt-2"
              onClick={() => { if (transcript.text) sendMessage(transcript.text); }}
            >
              Send to chat
            </button>
          )}
        </div>
      )}

      {error && (
        <div className="voice-panel__error">
          <span className="badge badge-error">Error</span>
          <p className="selectable">{error}</p>
        </div>
      )}

      {/* Options */}
      <div className="voice-panel__options">
        <label className="voice-panel__option">
          <input
            type="checkbox"
            checked={insertOnDone}
            onChange={(e) => setInsertOnDone(e.target.checked)}
          />
          <span>Auto-send transcript to chat</span>
        </label>
      </div>

      {/* Models */}
      {modelsLoaded && (
        <div className="voice-panel__models">
          <h3 className="voice-panel__section-title">Available models</h3>
          <div className="voice-panel__model-list">
            {models.map((m) => (
              <div key={m.id} className="voice-panel__model-row">
                <div className="flex items-center gap-2">
                  <span className={`status-dot ${m.downloaded ? "online" : "offline"}`} />
                  <span className="voice-panel__model-name">{m.name}</span>
                  {m.recommended && <span className="badge badge-info">Recommended</span>}
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-secondary text-xs">{m.sizeMb >= 1000 ? `${(m.sizeMb/1000).toFixed(1)} GB` : `${m.sizeMb} MB`}</span>
                  {!m.downloaded && (
                    <span className="badge badge-neutral">Not downloaded</span>
                  )}
                </div>
              </div>
            ))}
          </div>
          <p className="text-secondary text-xs mt-2">
            Run <code>hermes install-models</code> to download models to ~/.hermes/models/
          </p>
        </div>
      )}

      <style>{voiceStyles}</style>
    </div>
  );
}

function RecordButton({
  recordingState,
  onStart,
  onStop,
  disabled,
}: {
  recordingState:  RecordingState;
  onStart:         () => void;
  onStop:          () => void;
  disabled:        boolean;
}) {
  const isRecording = recordingState === "recording";
  const isProcessing = recordingState === "processing";

  return (
    <button
      className={`voice-btn ${isRecording ? "voice-btn--recording" : ""} ${isProcessing ? "voice-btn--processing" : ""}`}
      onClick={isRecording ? onStop : onStart}
      disabled={disabled || isProcessing}
    >
      {isProcessing ? (
        <span className="voice-btn__icon">⟳</span>
      ) : isRecording ? (
        <span className="voice-btn__icon">■</span>
      ) : (
        <span className="voice-btn__icon">⏺</span>
      )}
    </button>
  );
}

function fmtTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = (seconds % 60).toFixed(1);
  return `${m}:${s.padStart(4, "0")}`;
}

const voiceStyles = `
  .voice-panel {
    display: flex;
    flex-direction: column;
    gap: var(--sp-6);
    padding: var(--sp-6);
    max-width: 640px;
    margin: 0 auto;
    width: 100%;
    overflow-y: auto;
    height: 100%;
  }

  .voice-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .voice-panel__title {
    font-size: var(--text-xl);
    font-weight: var(--weight-semi);
  }

  .voice-panel__status {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .voice-panel__record-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-4);
    padding: var(--sp-8) 0;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
  }

  .voice-btn {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    background: var(--surface-2);
    border: 2px solid var(--border-2);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 150ms ease;
    cursor: pointer;
  }

  .voice-btn:hover:not(:disabled) {
    border-color: var(--accent);
    background: var(--accent-dim);
  }

  .voice-btn--recording {
    background: rgba(239,68,68,0.15);
    border-color: var(--error);
    animation: recording-pulse 1.2s ease-in-out infinite;
  }

  .voice-btn--processing {
    opacity: 0.5;
  }

  @keyframes recording-pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(239,68,68,0.4); }
    50%       { box-shadow: 0 0 0 12px rgba(239,68,68,0); }
  }

  .voice-btn__icon {
    font-size: 24px;
    color: var(--text-primary);
    line-height: 1;
  }

  .voice-btn--recording .voice-btn__icon { color: var(--error); }
  .voice-btn:hover:not(:disabled) .voice-btn__icon { color: var(--accent); }

  .voice-panel__record-hint {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .voice-panel__result {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .voice-panel__result-header {
    display: flex;
    gap: var(--sp-2);
    align-items: center;
  }

  .voice-panel__transcript {
    font-size: var(--text-base);
    line-height: 1.65;
    color: var(--text-primary);
    white-space: pre-wrap;
  }

  .voice-panel__segments {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .voice-panel__segment {
    display: flex;
    gap: var(--sp-3);
    padding: var(--sp-1) 0;
    border-top: 1px solid var(--border);
  }

  .voice-panel__segment-time {
    flex-shrink: 0;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    min-width: 100px;
  }

  .voice-panel__error {
    background: rgba(239,68,68,0.08);
    border: 1px solid rgba(239,68,68,0.3);
    border-radius: var(--radius);
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .voice-panel__options {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .voice-panel__option {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .voice-panel__models {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--sp-4);
  }

  .voice-panel__section-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    margin-bottom: var(--sp-3);
  }

  .voice-panel__model-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .voice-panel__model-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-2) 0;
    border-bottom: 1px solid var(--border);
  }

  .voice-panel__model-row:last-child {
    border-bottom: none;
  }

  .voice-panel__model-name {
    font-size: var(--text-sm);
    color: var(--text-primary);
  }
`;
