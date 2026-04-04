// components/MediaPanel.tsx — FFmpeg media processing panel

import React, { useState, useCallback } from "react";
import * as bridge from "../lib/tauri-bridge";
import type { MediaInfo, ProcessResult } from "../lib/types";

type Operation = "info" | "extract-audio" | "convert-whisper" | "transcode-audio" | "transcode-video";

export function MediaPanel() {
  const [inputPath, setInputPath]   = useState("");
  const [outputPath, setOutputPath] = useState("");
  const [operation, setOperation]   = useState<Operation>("info");
  const [mediaInfo, setMediaInfo]   = useState<MediaInfo | null>(null);
  const [result, setResult]         = useState<ProcessResult | null>(null);
  const [error, setError]           = useState<string | null>(null);
  const [processing, setProcessing] = useState(false);

  const pickFile = useCallback(async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const path = await open({
        multiple: false,
        filters: [
          { name: "Media", extensions: ["mp4", "mkv", "avi", "mov", "webm", "mp3", "wav", "flac", "ogg", "m4a", "aac"] },
        ],
      });
      if (typeof path === "string") {
        setInputPath(path);
        setMediaInfo(null);
        setResult(null);
        setError(null);

        // Auto-probe
        try {
          const info = await bridge.getMediaInfo(path);
          setMediaInfo(info);
        } catch {
          // Non-fatal — user can still process
        }
      }
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const pickOutput = useCallback(async () => {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const extMap: Record<Operation, string[]> = {
        "info":            [],
        "extract-audio":   ["wav"],
        "convert-whisper": ["wav"],
        "transcode-audio": ["mp3", "wav", "flac", "ogg", "aac"],
        "transcode-video": ["mp4", "mkv", "webm"],
      };
      const exts = extMap[operation];
      const path = await save({
        filters: exts.length ? [{ name: "Output", extensions: exts }] : [],
      });
      if (path) setOutputPath(path);
    } catch (err) {
      setError(String(err));
    }
  }, [operation]);

  const handleProcess = useCallback(async () => {
    if (!inputPath) return;
    setProcessing(true);
    setError(null);
    setResult(null);

    try {
      let res: ProcessResult | null = null;
      switch (operation) {
        case "info": {
          const info = await bridge.getMediaInfo(inputPath);
          setMediaInfo(info);
          setProcessing(false);
          return;
        }
        case "extract-audio":
          res = await bridge.extractAudioFromVideo(inputPath, outputPath || undefined);
          break;
        case "convert-whisper":
          res = await bridge.convertAudioForWhisper(inputPath, outputPath || undefined);
          break;
        case "transcode-audio":
          if (!outputPath) {
            setError("Output path required for transcoding");
            setProcessing(false);
            return;
          }
          res = await bridge.processAudio({ inputPath, outputPath });
          break;
        case "transcode-video":
          if (!outputPath) {
            setError("Output path required for transcoding");
            setProcessing(false);
            return;
          }
          res = await bridge.processAudio({ inputPath, outputPath });
          break;
      }
      if (res) setResult(res);
    } catch (err) {
      setError(String(err));
    } finally {
      setProcessing(false);
    }
  }, [inputPath, outputPath, operation]);

  const opOptions: { value: Operation; label: string }[] = [
    { value: "info",            label: "Probe file info" },
    { value: "extract-audio",   label: "Extract audio from video" },
    { value: "convert-whisper", label: "Convert to Whisper format (16kHz WAV)" },
    { value: "transcode-audio", label: "Transcode audio" },
    { value: "transcode-video", label: "Transcode video" },
  ];

  return (
    <div className="media-panel">
      <div className="media-panel__header">
        <h2 className="media-panel__title">Media Processing</h2>
        <span className="text-secondary text-sm">Powered by FFmpeg</span>
      </div>

      {/* Input */}
      <div className="media-panel__field">
        <label className="media-panel__label">Input file</label>
        <div className="media-panel__path-row">
          <input
            className="input media-panel__path-input"
            value={inputPath}
            onChange={(e) => setInputPath(e.target.value)}
            placeholder="/path/to/media.mp4"
          />
          <button className="btn btn-ghost" onClick={pickFile}>Browse</button>
        </div>
      </div>

      {/* Operation */}
      <div className="media-panel__field">
        <label className="media-panel__label">Operation</label>
        <select
          className="input"
          value={operation}
          onChange={(e) => setOperation(e.target.value as Operation)}
        >
          {opOptions.map((op) => (
            <option key={op.value} value={op.value}>{op.label}</option>
          ))}
        </select>
      </div>

      {/* Output (only for processing operations) */}
      {operation !== "info" && (
        <div className="media-panel__field">
          <label className="media-panel__label">Output path <span className="text-secondary">(optional — auto-generated if empty)</span></label>
          <div className="media-panel__path-row">
            <input
              className="input media-panel__path-input"
              value={outputPath}
              onChange={(e) => setOutputPath(e.target.value)}
              placeholder="Leave empty to auto-generate"
            />
            <button className="btn btn-ghost" onClick={pickOutput}>Save as…</button>
          </div>
        </div>
      )}

      <button
        className="btn btn-primary"
        onClick={handleProcess}
        disabled={!inputPath || processing}
        style={{ alignSelf: "flex-start" }}
      >
        {processing ? "Processing…" : operation === "info" ? "Probe" : "Process"}
      </button>

      {/* Media info */}
      {mediaInfo && (
        <div className="media-panel__info">
          <h3 className="media-panel__section-title">File info</h3>
          <div className="info-grid">
            <InfoRow label="Format"   value={mediaInfo.format} />
            <InfoRow label="Duration" value={mediaInfo.durationS ? `${mediaInfo.durationS.toFixed(2)}s` : "—"} />
            <InfoRow label="Size"     value={fmtSize(mediaInfo.sizeBytes)} />
            <InfoRow label="Bitrate"  value={mediaInfo.bitrateKbps ? `${mediaInfo.bitrateKbps} kbps` : "—"} />
            {mediaInfo.videoCodec && (
              <InfoRow label="Video" value={`${mediaInfo.videoCodec} ${mediaInfo.width}×${mediaInfo.height} @ ${mediaInfo.fps?.toFixed(2)} fps`} />
            )}
            {mediaInfo.audioCodec && (
              <InfoRow label="Audio" value={`${mediaInfo.audioCodec} ${mediaInfo.sampleRate} Hz · ${mediaInfo.channels}ch`} />
            )}
          </div>
        </div>
      )}

      {/* Process result */}
      {result && (
        <div className="media-panel__result">
          <span className="badge badge-success">Done</span>
          <div className="info-grid mt-2">
            <InfoRow label="Output"   value={result.outputPath} mono />
            <InfoRow label="Size"     value={fmtSize(result.sizeBytes)} />
            <InfoRow label="Duration" value={`${(result.durationMs / 1000).toFixed(2)}s`} />
          </div>
        </div>
      )}

      {error && (
        <div className="media-panel__error">
          <span className="badge badge-error">Error</span>
          <p className="selectable text-sm">{error}</p>
        </div>
      )}

      <style>{mediaStyles}</style>
    </div>
  );
}

function InfoRow({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="info-row">
      <span className="info-row__label">{label}</span>
      <span className={`info-row__value selectable ${mono ? "mono" : ""}`}>{value}</span>
    </div>
  );
}

function fmtSize(bytes: number): string {
  if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(2)} GB`;
  if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(1)} MB`;
  if (bytes >= 1e3) return `${(bytes / 1e3).toFixed(0)} KB`;
  return `${bytes} B`;
}

const mediaStyles = `
  .media-panel {
    display: flex;
    flex-direction: column;
    gap: var(--sp-5);
    padding: var(--sp-6);
    max-width: 680px;
    margin: 0 auto;
    width: 100%;
    overflow-y: auto;
    height: 100%;
  }

  .media-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .media-panel__title {
    font-size: var(--text-xl);
    font-weight: var(--weight-semi);
  }

  .media-panel__field {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .media-panel__label {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .media-panel__path-row {
    display: flex;
    gap: var(--sp-2);
  }

  .media-panel__path-input {
    flex: 1;
  }

  .media-panel__info,
  .media-panel__result {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .media-panel__section-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .media-panel__error {
    background: rgba(239,68,68,0.08);
    border: 1px solid rgba(239,68,68,0.3);
    border-radius: var(--radius);
    padding: var(--sp-3) var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .info-grid {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .info-row {
    display: flex;
    gap: var(--sp-4);
    padding: 4px 0;
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
    word-break: break-all;
  }
`;
