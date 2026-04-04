// lib/types.ts — Shared TypeScript types for Hermes Foundry Desktop

export type Panel = "chat" | "voice" | "media" | "agent" | "settings";

export interface Message {
  id:              string;
  role:            "user" | "assistant" | "system" | "tool";
  content:         string;
  model?:          string;
  conversationId:  string;
  timestamp:       number;
  streaming?:      boolean;
  toolCalls?:      ToolCall[];
  error?:          string;
}

export interface ToolCall {
  id:       string;
  name:     string;
  input:    Record<string, unknown>;
  output?:  string;
  status:   "pending" | "running" | "done" | "error";
}

export interface Conversation {
  id:           string;
  title:        string;
  model:        string;
  messageCount: number;
  createdAt:    number;
  updatedAt:    number;
}

export interface ServiceStatus {
  name:    string;
  status:  "running" | "stopped" | "error" | "starting";
  url:     string;
  healthy: boolean;
}

export interface AllServicesStatus {
  hermes_api:     ServiceStatus;
  llama_server:   ServiceStatus;
  whisper_server: ServiceStatus;
}

export interface LocalModel {
  id:             string;
  name:           string;
  path:           string;
  sizeGb?:        number;
  quantization?:  string;
  loaded:         boolean;
  contextLength?: number;
}

export interface VoiceModel {
  id:          string;
  name:        string;
  sizeMb:      number;
  downloaded:  boolean;
  recommended: boolean;
}

export interface TranscriptionResult {
  text:       string;
  language?:  string;
  durationS?: number;
  segments:   TranscriptionSegment[];
}

export interface TranscriptionSegment {
  id:    number;
  start: number;
  end:   number;
  text:  string;
}

export interface MediaInfo {
  durationS?:   number;
  format:       string;
  sizeBytes:    number;
  videoCodec?:  string;
  audioCodec?:  string;
  width?:       number;
  height?:      number;
  fps?:         number;
  bitrateKbps?: number;
  sampleRate?:  number;
  channels?:    number;
}

export interface ProcessResult {
  outputPath: string;
  durationMs: number;
  sizeBytes:  number;
}

export interface Branding {
  app_name:        string;
  app_id:          string;
  window_title:    string;
  company_name:    string;
  support_url:     string;
  welcome_message: string;
  colors: {
    accent:      string;
    bg:          string;
    surface:     string;
    border:      string;
    text_primary: string;
  };
  default_model?:       string;
  default_voice_model?: string;
}

export interface DashboardOverview {
  total_runs:        number;
  active_runs:       number;
  pending_approvals: number;
  total_tools:       number;
  agents_online:     number;
}

export interface AgentRun {
  id:         string;
  status:     "running" | "completed" | "failed" | "pending";
  model:      string;
  task:       string;
  started_at: string;
  ended_at?:  string;
  tool_calls: number;
}

export interface Approval {
  id:          string;
  run_id:      string;
  tool_name:   string;
  tool_input:  Record<string, unknown>;
  reason:      string;
  created_at:  string;
}
