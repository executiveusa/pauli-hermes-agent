import * as fs from 'fs';
import * as path from 'path';
import YAML from 'yaml';
import {
  SandcastleProvider,
  SandcastleRun,
  SandcastleRunInput,
  SandcastleRunResult,
  SandcastleEvent,
  SandcastleHealth,
} from '@/lib/schemas/sandcastle';

interface SandcastleConfig {
  default_sandbox_policy: {
    enabled: boolean;
    default_for_coding_tasks: boolean;
    direct_host_execution_allowed: boolean;
    no_sandbox_allowed: boolean;
    require_branch_per_bead: boolean;
    require_tests_before_merge: boolean;
    require_human_approval_before_merge: boolean;
  };
  providers: {
    preferred_order: string[];
    [key: string]: any;
  };
  limits: {
    max_parallel_sandboxes: number;
    default_max_iterations: number;
    idle_timeout_seconds: number;
    max_runtime_minutes: number;
    max_log_bytes: number;
  };
  observability: {
    stream_events: boolean;
    persist_logs: boolean;
    persist_commits: boolean;
  };
}

let config: SandcastleConfig;
let runs: Map<string, SandcastleRun> = new Map();
let events: Map<string, SandcastleEvent[]> = new Map();

function loadConfig(): SandcastleConfig {
  const configPath = path.join(process.cwd(), 'config', 'pauli_sandbox_registry.yaml');
  if (!fs.existsSync(configPath)) {
    throw new Error(`Sandcastle config not found: ${configPath}`);
  }
  const content = fs.readFileSync(configPath, 'utf-8');
  return YAML.parse(content);
}

function getConfig(): SandcastleConfig {
  if (!config) {
    config = loadConfig();
  }
  return config;
}

export async function listSandcastleProviders(): Promise<SandcastleProvider[]> {
  const cfg = getConfig();
  const providers: SandcastleProvider[] = [];

  for (const providerKey of cfg.providers.preferred_order) {
    const providerConfig = cfg.providers[providerKey];
    if (!providerConfig) continue;

    const enabled = process.env[providerConfig.enabled_env] !== 'false';
    let healthy = enabled;
    const missingRequirements: string[] = [];

    if (providerConfig.required_env) {
      for (const envVar of providerConfig.required_env) {
        if (!process.env[envVar]) {
          healthy = false;
          missingRequirements.push(`${envVar} environment variable`);
        }
      }
    }

    if (providerConfig.required_binaries) {
      for (const binary of providerConfig.required_binaries) {
        // In real implementation, check if binary exists
        if (!process.env[providerConfig.enabled_env]) {
          healthy = false;
          missingRequirements.push(`${binary} binary`);
        }
      }
    }

    providers.push({
      id: providerKey,
      label: providerConfig.type === 'firecracker_microvm' ? '🚀 Vercel Firecracker' :
             providerConfig.type === 'local_container' ? '🐳 Docker' : '📦 Podman',
      type: providerConfig.type,
      enabled,
      healthy,
      missingRequirements: missingRequirements.length > 0 ? missingRequirements : undefined,
      description: providerConfig.description,
    });
  }

  return providers;
}

export async function healthcheckSandcastle(): Promise<SandcastleHealth> {
  const cfg = getConfig();
  const providers = await listSandcastleProviders();
  const healthyProviders = providers.filter(p => p.healthy);

  return {
    healthy: healthyProviders.length > 0 && cfg.default_sandbox_policy.enabled,
    providers,
    activeRuns: Array.from(runs.values()).filter(r =>
      r.status === 'running' || r.status === 'waiting_for_agent' || r.status === 'tests_running'
    ).length,
    totalRuns: runs.size,
  };
}

async function selectProvider(preferredId?: string): Promise<SandcastleProvider> {
  const providers = await listSandcastleProviders();

  if (preferredId) {
    const preferred = providers.find(p => p.id === preferredId);
    if (preferred?.healthy) return preferred;
  }

  const healthyProvider = providers.find(p => p.healthy);
  if (!healthyProvider) {
    throw new Error('No healthy Sandcastle provider available. Check environment configuration.');
  }

  return healthyProvider;
}

function emitEvent(
  runId: string,
  type: SandcastleEvent['type'],
  message: string,
  metadata?: Record<string, any>,
  severity: 'info' | 'warn' | 'error' = 'info'
): void {
  if (!events.has(runId)) {
    events.set(runId, []);
  }

  const event: SandcastleEvent = {
    id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
    runId,
    type,
    timestamp: Date.now(),
    message,
    metadata,
    severity,
  };

  events.get(runId)!.push(event);

  // Persist if configured
  const cfg = getConfig();
  if (cfg.observability.persist_logs) {
    // In real implementation, persist to database or file
    console.log(`[${runId}] ${type}: ${message}`);
  }
}

function redactSecrets(text: string): string {
  const patterns = [
    /OPENAI_API_KEY=[\w\-]+/g,
    /Authorization: Bearer [\w\-\.]+/g,
    /token=[\w\-]+/g,
  ];

  let redacted = text;
  for (const pattern of patterns) {
    redacted = redacted.replace(pattern, '[REDACTED]');
  }
  return redacted;
}

export async function runSandcastleTask(input: SandcastleRunInput): Promise<SandcastleRunResult> {
  const cfg = getConfig();
  const provider = await selectProvider(input.providerId);

  const runId = `run-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
  const branch = `agent/${input.beadId}-${input.title.toLowerCase().replace(/\s+/g, '-')}`;

  const run: SandcastleRun = {
    id: runId,
    beadId: input.beadId,
    title: input.title,
    prompt: input.prompt,
    status: 'queued',
    provider: provider.id,
    branch,
    agent: input.agent,
    modelRoute: input.modelRoute,
    startedAt: Date.now(),
    commits: [],
    changedFiles: [],
    approvalStatus: input.requireApproval ? 'pending' : undefined,
  };

  runs.set(runId, run);
  events.set(runId, []);

  emitEvent(runId, 'provider_selected', `Selected provider: ${provider.label}`);
  emitEvent(runId, 'branch_created', `Created branch: ${branch}`);

  // Simulate sandbox startup
  run.status = 'preparing';
  emitEvent(runId, 'sandbox_starting', 'Starting sandbox...');

  await new Promise(r => setTimeout(r, 500));

  run.status = 'sandbox_starting';
  emitEvent(runId, 'sandbox_created', `Sandbox created on ${provider.label}`);

  // Simulate agent execution
  run.status = 'running';
  emitEvent(runId, 'agent_started', `Agent started: ${input.agent || 'default'}`);

  // Simulate work
  await new Promise(r => setTimeout(r, 1000));

  emitEvent(runId, 'file_changed', 'Modified: src/app.tsx');
  emitEvent(runId, 'file_changed', 'Modified: lib/utils.ts');

  // Simulate test execution
  if (input.requireTests !== false) {
    run.status = 'tests_running';
    emitEvent(runId, 'test_started', 'Running test suite...');

    await new Promise(r => setTimeout(r, 500));

    run.testsPassed = true;
    emitEvent(runId, 'test_finished', 'Tests passed: 42 / 42');
  }

  // Simulate commit
  run.commits.push({
    sha: 'abc1234',
    message: input.title,
    author: 'Hermes Agent',
    timestamp: Date.now(),
  });
  emitEvent(runId, 'commit_created', `Committed: ${input.title}`);

  run.changedFiles = ['src/app.tsx', 'lib/utils.ts'];
  run.status = input.requireApproval ? 'needs_approval' : 'completed';
  run.endedAt = Date.now();
  run.durationMs = run.endedAt - run.startedAt;

  if (input.requireApproval) {
    emitEvent(runId, 'approval_required', 'Awaiting human approval before merge');
  } else {
    run.approvalStatus = 'approved';
    run.status = 'completed';
    emitEvent(runId, 'merge_approved', 'Auto-approved (no approval required)');
  }

  return {
    run,
    events: events.get(runId) || [],
    success: true,
  };
}

export async function stopSandcastleRun(runId: string): Promise<void> {
  const run = runs.get(runId);
  if (!run) {
    throw new Error(`Run not found: ${runId}`);
  }

  run.status = 'stopped';
  run.endedAt = Date.now();
  run.durationMs = run.endedAt - run.startedAt;

  emitEvent(runId, 'error', 'Sandbox stopped by user', {}, 'warn');
}

export async function retrySandcastleRun(runId: string): Promise<SandcastleRunResult> {
  const originalRun = runs.get(runId);
  if (!originalRun) {
    throw new Error(`Run not found: ${runId}`);
  }

  return runSandcastleTask({
    beadId: originalRun.beadId,
    title: originalRun.title,
    prompt: originalRun.prompt,
    agent: originalRun.agent,
    modelRoute: originalRun.modelRoute,
  });
}

export async function discardSandcastleRun(runId: string): Promise<void> {
  const run = runs.get(runId);
  if (!run) {
    throw new Error(`Run not found: ${runId}`);
  }

  run.status = 'discarded';
  emitEvent(runId, 'sandbox_discarded', `Sandbox discarded. Branch ${run.branch} can be deleted.`);
}

export function getSandcastleRun(runId: string): SandcastleRun | undefined {
  return runs.get(runId);
}

export function listSandcastleRuns(): SandcastleRun[] {
  return Array.from(runs.values());
}

export function getSandcastleEvents(runId: string): SandcastleEvent[] {
  return events.get(runId) || [];
}
