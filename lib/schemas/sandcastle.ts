import { z } from 'zod';

export const SandcastleProviderTypeSchema = z.enum([
  'firecracker_microvm',
  'local_container',
  'local_rootless_container',
  'no_sandbox',
]);

export const SandcastleProviderSchema = z.object({
  id: z.string(),
  label: z.string(),
  type: SandcastleProviderTypeSchema,
  enabled: z.boolean(),
  healthy: z.boolean(),
  missingRequirements: z.array(z.string()).optional(),
  description: z.string().optional(),
  latency_ms: z.number().optional(),
});

export const SandcastleRunStatusSchema = z.enum([
  'queued',
  'preparing',
  'sandbox_starting',
  'running',
  'waiting_for_agent',
  'tests_running',
  'review_running',
  'completed',
  'failed',
  'stopped',
  'needs_approval',
  'discarded',
]);

export const SandcastleEventTypeSchema = z.enum([
  'provider_selected',
  'branch_created',
  'sandbox_created',
  'prompt_loaded',
  'agent_started',
  'agent_stream_text',
  'tool_call',
  'file_changed',
  'command_started',
  'command_finished',
  'test_started',
  'test_finished',
  'commit_created',
  'review_started',
  'review_finished',
  'approval_required',
  'merge_approved',
  'sandbox_discarded',
  'error',
]);

export const SandcastleEventSchema = z.object({
  id: z.string(),
  runId: z.string(),
  type: SandcastleEventTypeSchema,
  timestamp: z.number(),
  message: z.string(),
  metadata: z.record(z.any()).optional(),
  severity: z.enum(['info', 'warn', 'error']).default('info'),
});

export const SandcastleCommitSchema = z.object({
  sha: z.string(),
  message: z.string(),
  author: z.string(),
  timestamp: z.number(),
});

export const SandcastleRunSchema = z.object({
  id: z.string(),
  beadId: z.string(),
  title: z.string(),
  prompt: z.string(),
  status: SandcastleRunStatusSchema,
  provider: z.string(),
  branch: z.string(),
  agent: z.string().optional(),
  modelRoute: z.string().optional(),
  startedAt: z.number(),
  endedAt: z.number().optional(),
  durationMs: z.number().optional(),
  commits: z.array(SandcastleCommitSchema),
  changedFiles: z.array(z.string()),
  logsUrl: z.string().optional(),
  testsPassed: z.boolean().optional(),
  approvalStatus: z.enum(['pending', 'approved', 'rejected']).optional(),
  error: z.string().optional(),
});

export const SandcastleRunInputSchema = z.object({
  beadId: z.string(),
  title: z.string(),
  prompt: z.string(),
  agent: z.string().optional(),
  modelRoute: z.string().optional(),
  maxIterations: z.number().default(5),
  requireTests: z.boolean().default(true),
  requireApproval: z.boolean().default(true),
  providerId: z.string().optional(),
});

export const SandcastleRunResultSchema = z.object({
  run: SandcastleRunSchema,
  events: z.array(SandcastleEventSchema),
  success: z.boolean(),
});

export const SandcastleHealthSchema = z.object({
  healthy: z.boolean(),
  providers: z.array(SandcastleProviderSchema),
  activeRuns: z.number(),
  totalRuns: z.number(),
});

export const ApprovalSchema = z.object({
  id: z.string(),
  runId: z.string(),
  type: z.enum(['merge', 'deploy', 'paid_model', 'destructive_action']),
  status: z.enum(['pending', 'approved', 'rejected']),
  requestedAt: z.number(),
  reviewedAt: z.number().optional(),
  reviewedBy: z.string().optional(),
  reason: z.string().optional(),
});

export type SandcastleProvider = z.infer<typeof SandcastleProviderSchema>;
export type SandcastleRun = z.infer<typeof SandcastleRunSchema>;
export type SandcastleRunInput = z.infer<typeof SandcastleRunInputSchema>;
export type SandcastleRunResult = z.infer<typeof SandcastleRunResultSchema>;
export type SandcastleRunStatus = z.infer<typeof SandcastleRunStatusSchema>;
export type SandcastleEvent = z.infer<typeof SandcastleEventSchema>;
export type SandcastleHealth = z.infer<typeof SandcastleHealthSchema>;
export type Approval = z.infer<typeof ApprovalSchema>;
