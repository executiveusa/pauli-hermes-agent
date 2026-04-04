// lib/tauri-bridge.ts — Thin typed wrappers around Tauri invoke calls

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type {
  Message,
  LocalModel,
  VoiceModel,
  TranscriptionResult,
  MediaInfo,
  ProcessResult,
  AllServicesStatus,
  DashboardOverview,
  AgentRun,
  Approval,
  Branding,
} from "./types";

// ── Chat ─────────────────────────────────────────────────────────────────────

export interface StreamCallbacks {
  onToken:  (conversationId: string, token: string) => void;
  onDone:   (conversationId: string) => void;
  onError:  (conversationId: string, error: string) => void;
}

export async function sendMessage(params: {
  message:        string;
  conversationId?: string;
  model?:         string;
  useLocal?:      boolean;
  systemPrompt?:  string;
  temperature?:   number;
  maxTokens?:     number;
}): Promise<{ content: string; conversationId: string; modelUsed: string }> {
  return invoke("send_message", {
    request: {
      message:         params.message,
      conversation_id: params.conversationId,
      model:           params.model,
      use_local:       params.useLocal,
      system_prompt:   params.systemPrompt,
      temperature:     params.temperature,
      max_tokens:      params.maxTokens,
    },
  });
}

export async function streamMessage(
  params: {
    message:        string;
    conversationId?: string;
    model?:         string;
    useLocal?:      boolean;
    temperature?:   number;
    maxTokens?:     number;
  },
  callbacks: StreamCallbacks
): Promise<{ unlisten: UnlistenFn }> {
  const unlistenToken = await listen<{ conversation_id: string; token: string }>(
    "chat:token",
    (event) => callbacks.onToken(event.payload.conversation_id, event.payload.token)
  );
  const unlistenDone = await listen<{ conversation_id: string }>(
    "chat:done",
    (event) => callbacks.onDone(event.payload.conversation_id)
  );
  const unlistenError = await listen<{ conversation_id: string; error: string }>(
    "chat:error",
    (event) => callbacks.onError(event.payload.conversation_id, event.payload.error)
  );

  await invoke("stream_message", {
    request: {
      message:         params.message,
      conversation_id: params.conversationId,
      model:           params.model,
      use_local:       params.useLocal,
      temperature:     params.temperature,
      max_tokens:      params.maxTokens,
    },
  });

  const unlisten = () => {
    unlistenToken();
    unlistenDone();
    unlistenError();
  };
  return { unlisten };
}

export async function stopGeneration(): Promise<void> {
  return invoke("stop_generation");
}

// ── Voice ────────────────────────────────────────────────────────────────────

export async function transcribeFile(
  filePath: string,
  language?: string
): Promise<TranscriptionResult> {
  return invoke("transcribe_file", { filePath, language });
}

export async function listVoiceModels(): Promise<VoiceModel[]> {
  return invoke("list_voice_models");
}

export async function getVoiceServerStatus(): Promise<{ ready: boolean; url: string }> {
  return invoke("get_voice_server_status");
}

// ── Local LLM ────────────────────────────────────────────────────────────────

export async function listLocalModels(): Promise<LocalModel[]> {
  return invoke("list_local_models");
}

export async function getLlmServerStatus(): Promise<{
  ready: boolean;
  loadedModel?: string;
  url: string;
}> {
  return invoke("get_llm_server_status");
}

export async function loadLocalModel(modelPath: string): Promise<void> {
  return invoke("load_local_model", { modelPath });
}

export async function scanModelsDir(): Promise<{
  modelCount:  number;
  totalSizeGb: number;
  modelsDir:   string;
}> {
  return invoke("scan_models_dir");
}

// ── Media ────────────────────────────────────────────────────────────────────

export async function getMediaInfo(filePath: string): Promise<MediaInfo> {
  return invoke("get_media_info", { filePath });
}

export async function extractAudioFromVideo(
  inputPath: string,
  outputPath?: string
): Promise<ProcessResult> {
  return invoke("extract_audio_from_video", { inputPath, outputPath });
}

export async function convertAudioForWhisper(
  inputPath: string,
  outputPath?: string
): Promise<ProcessResult> {
  return invoke("convert_audio_for_whisper", { inputPath, outputPath });
}

export async function processAudio(params: {
  inputPath:   string;
  outputPath:  string;
  format?:     string;
  sampleRate?: number;
  channels?:   number;
}): Promise<ProcessResult> {
  return invoke("process_audio", params);
}

// ── Services ─────────────────────────────────────────────────────────────────

export async function getAllServicesStatus(): Promise<AllServicesStatus> {
  return invoke("get_all_services_status");
}

export async function getDashboardOverview(): Promise<DashboardOverview> {
  return invoke("get_dashboard_overview");
}

export async function getRuns(limit?: number, status?: string): Promise<{ runs: AgentRun[] }> {
  return invoke("get_runs", { limit, status });
}

export async function getApprovals(): Promise<{ approvals: Approval[] }> {
  return invoke("get_approvals");
}

export async function approveCommand(approvalId: string): Promise<void> {
  return invoke("approve_command", { approvalId });
}

export async function rejectCommand(approvalId: string, reason?: string): Promise<void> {
  return invoke("reject_command", { approvalId, reason });
}

// ── Settings ─────────────────────────────────────────────────────────────────

export async function saveApiKey(service: string, apiKey: string): Promise<void> {
  return invoke("save_api_key", { service, apiKey });
}

export async function getApiKey(service: string, masked?: boolean): Promise<string | null> {
  return invoke("get_api_key", { service, masked });
}

export async function getBranding(): Promise<Branding> {
  return invoke("get_branding");
}
